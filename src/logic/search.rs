use crate::data::consts::{GENRES, LOSSLESS_COMPRESSED_FORMATS, LOSSLESS_UNCOMPRESSED_FORMATS, LOSSY_FORMATS, MAX_QUANTITY};
use crate::data::dictionary;
use crate::data::dictionary::Lang;
use crate::db::dbcore::Database;
use crate::log;
use crate::logic::dsl::{Command, Data};
use anyhow::anyhow;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

pub struct LinkParams<'a> {
    pub form: &'a str,
    pub genre: &'a str,
    pub quantity: usize,
    pub period: bool,
    pub lang: Lang,
    pub file_history: bool,
    pub lossless: bool,
    pub client: reqwest::Client,
    pub db: Option<Database>,
    pub tx1: Arc<RwLock<UnboundedSender<Data>>>,
}

pub struct Link;
impl Link {
    pub async fn get_all_links<'a>(link_params: LinkParams<'a>) -> anyhow::Result<Option<Vec<String>>> {

        // 1. Get the link set
        let mut found_links: HashSet<String> = HashSet::new();
        let mut page_number = 1;

        while (found_links.len() < link_params.quantity && !link_params.period) ||
              (found_links.len() < MAX_QUANTITY && link_params.period)
        {
            // If we found nothing on this page, stop searching
            if page_number > 1 && found_links.is_empty() { break; }

            let page = Page::new(
                page_number,
                link_params.form,
                link_params.genre,
                link_params.quantity,
                link_params.lossless,
                link_params.period,
                link_params.client.clone(),
            ).await;

            let found_links_on_page =
                match page.get_raw_page().await? {
                    Some(raw_page) => {
                        let html = Html::parse_document(&raw_page);
                        Self.get_filtered_links(&html, link_params.lossless)?
                    }
                    None => {
                        let msg = dictionary::errors::no_links_to_filtering(link_params.lang);
                        let data = Data::new(Command::Message, msg);
                        link_params.tx1.read().await.send(data)?;
                        return Ok(None);
                    }
                };

            let data = Data::new(Command::Search, page_number % 5);
            link_params.tx1.read().await.send(data)?;

            if !found_links_on_page.is_empty() {
                found_links.extend(found_links_on_page);
            }
            // If we found nothing on this page, stop searching
            else { break; }

            page_number += 1;

            // This is for safe scanning
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // 2. Remove duplicates
        //    Convert {"1.wav", "1.flac", "2.flac", "2.wav"} to {'1.flac', '2.flac'}
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
            let msg = dictionary::errors::no_links_to_filtering(link_params.lang);
            let data = Data::new(Command::Message, msg);
            link_params.tx1.read().await.send(data)?;
            return Ok(None);
        }

        // 3. Check found links in the history
        if link_params.file_history && let Some(db) = link_params.db {
            db.filter_by_history(&mut unique_links).await;
        }

        let mut found_links = unique_links.iter()
            .map(|(name, ext)| { format!("{name}.{ext}") })
            .collect::<Vec<String>>();

        // 4. Truncate found links
        if link_params.period { found_links.truncate(MAX_QUANTITY) }
        else { found_links.truncate(link_params.quantity) };

        if found_links.is_empty() {
            let msg = dictionary::errors::no_links_to_download(link_params.lang);
            let data = Data::new(Command::Message, msg);
            link_params.tx1.read().await.send(data)?;
            return Ok(None);
        }

        Ok(Some(found_links))
    }

    pub fn get_filtered_links(&self, raw_html: &Html, lossless: bool) -> anyhow::Result<HashSet<String>> {
        let formats: Vec<&str> = if lossless {
            LOSSLESS_COMPRESSED_FORMATS.iter().chain(&LOSSLESS_UNCOMPRESSED_FORMATS).copied().collect()
        } else {
            LOSSY_FORMATS.to_vec()
        };

        let selector = match Selector::parse("a") {
            Ok(s) => s,
            Err(e) => { return Err(anyhow!("{e}")) }
        };
        let mut links: HashSet<String> = HashSet::new();
        let elements = raw_html.select(&selector);
        for element in elements {
            if let Some(href) = element.value().attr("href") {
                let format_matches = formats.iter().any(|f| href.ends_with(f));

                let source_condition =
                    if cfg!(feature = "test") { true }            // Always pass if 'test' is on
                    else { href.find("/source/") > Some(1) }; // Check required if 'test' is off

                if format_matches && source_condition {
                    links.insert(href.to_owned());
                }
            }
        }
        Ok(links)
    }
}

struct Page {
    link: String,
    client: reqwest::Client,
}
impl Page {
    async fn new(
        number: usize,
        form: &str,
        genre: &str,
        quantity: usize,
        lossless: bool,
        period: bool,
        client: reqwest::Client,
    )
        -> Self
    {
        let bitrate = if lossless { "lossless" } else { "high" };
        let period =
            if period { format!("period=last&period_last={quantity}d&") }
            else { String::new() };

        let genre =
            if let Some((_, link_name)) = GENRES.iter().find(|(name, _)| name == &genre) { link_name }
            else { log!("GENRES parsing error"); "" };

        let link =
            if cfg!(feature = "test") { format!("http://localhost:80/{form}/{genre}?{period}bitrate={bitrate}&page={number}") }
            else { format!("https://promodj.com/{form}/{genre}?{period}bitrate={bitrate}&page={number}") };

        Self { link, client }
    }

    async fn get_raw_page(&self) -> anyhow::Result<Option<String>> {
        let response = self.client.get(&self.link).send().await?;
        if response.status() != 200 {
            log!("Bad status during parsing = {}", response.status());
            return Ok(None);
        }
        let raw_text = response.text().await?;
        Ok(Some(raw_text))
    }
}
