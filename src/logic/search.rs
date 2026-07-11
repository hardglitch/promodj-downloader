use crate::data::consts::{LOSSLESS_COMPRESSED_FORMATS, LOSSLESS_UNCOMPRESSED_FORMATS, LOSSY_FORMATS, MAX_QUANTITY};
use crate::data::dictionary;
use crate::log;
use crate::logic::dsl::{Command, Data};
use crate::ui::MyApp;
use anyhow::anyhow;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;
use crate::data::dictionary::Lang;
use crate::db::dbcore::Database;

pub struct Link;
impl Link {
    pub async fn get_all_links(
        form: &str,
        genre: &str,
        quantity: usize,
        period: bool,
        lang: Lang,
        file_history: bool,
        lossless: bool,
        client: reqwest::Client,
        db: Option<Database>,
        tx1: Arc<RwLock<Sender<Data>>>,
        rx2: Arc<RwLock<Receiver<Data>>>,
    )
        -> anyhow::Result<Option<Vec<String>>>
    {

        // 1. Get a raw link set
        let mut found_links: HashSet<String> = HashSet::new();
        let mut page_number = 1;

        while (found_links.len() < quantity && !period) ||
              (found_links.len() < MAX_QUANTITY && period)
        {
            // If we found nothing on this page, stop searching
            if page_number > 1 && found_links.is_empty() { break; }
            let raw_page = Self::get_raw_page(page_number, form, genre, quantity, lossless, period, client.clone()).await?;
            match raw_page {
                Some(raw_html) => {
                    let data = Data::new(Command::Temp, raw_html);
                    tx1.read().await.send(data).await?;
                }
                None => {
                    let msg = dictionary::errors::unable_to_connect(lang);
                    log!("{msg}");
                    let data = Data::new(Command::Message, msg);
                    tx1.read().await.send(data).await?;
                    return Ok(None);
                }
            };

            if let Some(data) = rx2.write().await.recv().await &&
               matches!(data.command(), Command::Temp) &&
               let Some(found_links_on_page) = data.payload::<HashSet<String>>()
            {
                if !found_links_on_page.is_empty() {
                    found_links.extend(found_links_on_page);
                } else {
                    // If we found nothing on this page, stop searching
                    break;
                }
            }
            let data = Data::new(Command::Search, page_number % 5);
            tx1.read().await.send(data).await?;

            page_number += 1;
        }

        // 2. Convert {"1.wav", "1.flac", "2.flac", "2.wav"} to {'1.flac', '2.flac'}
        let mut unique_links: HashMap<&str, &str> = HashMap::new();
        for link in &found_links {
            let mut split = link.rsplitn(2, '.');
            if let Some(ext) = split.next() &&
               let Some(name) = split.next()
            {
                unique_links
                    .entry(name)
                    .and_modify(|ext_| {
                        if LOSSLESS_UNCOMPRESSED_FORMATS.contains(ext_) &&
                           LOSSLESS_COMPRESSED_FORMATS.contains(&ext)
                        { *ext_ = ext }
                    })
                    .or_insert(ext);
            }
        }

        if unique_links.is_empty() {
            let msg = dictionary::errors::no_links_to_filtering(lang);
            let data = Data::new(Command::Message, msg);
            tx1.read().await.send(data).await?;
            return Ok(None);
        }

        // 3. Checking found links
        if file_history && let Some(db) = db {
            db.filter_by_history(&mut unique_links).await;
        }

        let mut found_links = unique_links.iter()
            .map(|(name, ext)| { format!("{name}.{ext}") })
            .collect::<Vec<String>>();

        // 4. Truncate found links
        if period { found_links.truncate(MAX_QUANTITY) }
        else { found_links.truncate(quantity) };

        if found_links.is_empty() {
            let msg = dictionary::errors::no_links_to_download(lang);
            let data = Data::new(Command::Message, msg);
            tx1.read().await.send(data).await?;
            return Ok(None);
        }

        Ok(Some(found_links))
    }

    // Calling outside
    pub fn parse(
        lossless: bool,
        tx2: Arc<RwLock<Sender<Data>>>,
        rx1: Arc<RwLock<Receiver<Data>>>,
    )
        -> anyhow::Result<()>
    {
        if let Ok(mut data) = rx1.try_write() &&
           let Ok(data) =  data.try_recv() &&
           matches!(data.command(), Command::Temp) &&
           let Some(raw_html) = data.payload::<String>()
        {
            let html = Html::parse_document(&raw_html);
            let found_links_on_page = Self.get_filtered_links(&html, lossless)?;
            let data = Data::new(Command::Temp, found_links_on_page);
            tx2.try_read()?.try_send(data)?;
        }
        Ok(())
    }

    async fn get_raw_page(
        page_number: usize,
        form: &str,
        genre: &str,
        quantity: usize,
        lossless: bool,
        period: bool,
        client: reqwest::Client,
    )
        -> anyhow::Result<Option<String>>
    {
        let page = Page::new(page_number, form, genre, quantity, lossless, period).await;
        page.get_raw_html(client).await
    }

    pub fn get_filtered_links(&self, link_massive: &Html, lossless: bool) -> anyhow::Result<HashSet<String>> {
        let formats: Vec<&str> = if lossless {
            LOSSLESS_COMPRESSED_FORMATS.iter().chain(&LOSSLESS_UNCOMPRESSED_FORMATS).copied().collect()
        } else {
            LOSSY_FORMATS.to_vec()
        };

        let selector = match Selector::parse("a[href]") {
            Ok(s) => s,
            Err(e) => { return Err(anyhow!("{e}")) }
        };
        let mut links: HashSet<String> = HashSet::new();
        let elements = link_massive.select(&selector);
        for element in elements {
            if let Some(href) = element.value().attr("href") &&
               formats.iter().any(|f| href.ends_with(f)) && href.find("/source/") > Some(1)
            {
                links.insert(href.to_owned());
            }
        }
        Ok(links)
    }
}

struct Page {
    link: String,
}
impl Page {
    async fn new(
        number: usize,
        form: &str,
        genre: &str,
        quantity: usize,
        lossless: bool,
        period: bool,
    )
        -> Self
    {
        let bitrate = if lossless { "lossless" } else { "high" };
        let period =
            if period { format!("period=last&period_last={quantity}d&") }
            else { String::new() };

        let link = format!("https://promodj.com/{form}/{genre}?{period}bitrate={bitrate}&page={number}");
        Self { link }
    }

    async fn get_raw_html(&self, client: reqwest::Client) -> anyhow::Result<Option<String>> {
        let response = client.get(&self.link).send().await?;
        if response.status() != 200 {
            log!("Bad status during parsing = {}", response.status());
            return Ok(None);
        }
        let text = response.text().await?;
        Ok(Some(text))
    }
}
