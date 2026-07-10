use crate::data::consts::{LOSSLESS_COMPRESSED_FORMATS, LOSSLESS_UNCOMPRESSED_FORMATS, LOSSY_FORMATS, MAX_QUANTITY};
use crate::data::dictionary;
use crate::log;
use crate::logic::dsl::{Command, Data};
use crate::ui::MyApp;
use anyhow::anyhow;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use tokio::sync::mpsc::Sender;

pub struct Link {
    tx: Sender<Data>,
}
impl Link {
    pub fn new(tx: Sender<Data>) -> Self {
        Self { tx }
    }

    pub async fn get_all_links<'a>(&mut self, app: &MyApp<'a>) -> anyhow::Result<Option<Vec<String>>> {

        // 1. Get a raw link set
        let mut found_links: HashSet<String> = HashSet::new();
        let mut page_number = 1;

        while (found_links.len() < app.quantity && !app.period) ||
              (found_links.len() < MAX_QUANTITY && app.period)
        {
            // If we found nothing on this page, stop searching
            if page_number > 1 && found_links.is_empty() { break; }

            let page = Page::new(page_number, app);
            let link_massive = page.parse(&app.client).await?;
            match link_massive {
                Some(html) => {
                    let found_links_on_page = self.get_filtered_links(&html, app.lossless)?;
                    if !found_links_on_page.is_empty() {
                        found_links.extend(found_links_on_page);
                    } else {
                        // If we found nothing on this page, stop searching
                        break;
                    }
                    let data = Data::new(Command::Search, page_number % 5);
                    self.tx.send(data).await?;
                }
                None => {
                    let msg = dictionary::errors::unable_to_connect(app.lang);
                    log!("{msg}");
                    let data = Data::new(Command::Message, msg);
                    self.tx.send(data).await?;
                    return Ok(None);
                }
            }
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
            let msg = dictionary::errors::no_links_to_filtering(app.lang);
            let data = Data::new(Command::Message, msg);
            self.tx.send(data).await?;
            return Ok(None);
        }

        // 3. Checking found links
        if app.file_history && let Some(db) = &app.db {
            db.filter_by_history(&mut unique_links).await;
        }

        let mut found_links = unique_links.iter()
            .map(|(name, ext)| { format!("{name}.{ext}") })
            .collect::<Vec<String>>();

        // 4. Truncate found links
        if app.period { found_links.truncate(MAX_QUANTITY) }
        else { found_links.truncate(app.quantity) };

        if found_links.is_empty() {
            let msg = dictionary::errors::no_links_to_download(app.lang);
            let data = Data::new(Command::Message, msg);
            self.tx.send(data).await?;
            return Ok(None);
        }

        Ok(Some(found_links))
    }

    fn get_filtered_links(&self, link_massive: &Html, lossless: bool) -> anyhow::Result<HashSet<String>> {
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
    fn new(number: usize, app: &MyApp) -> Self {
        let form = app.form;
        let genre = app.genre;
        let quantity = app.quantity;
        let bitrate = if app.lossless { "lossless" } else { "high" };
        let period =
            if app.period { format!("period=last&period_last={quantity}d&") }
            else { String::new() };

        let link = format!("https://promodj.com/{form}/{genre}?{period}bitrate={bitrate}&page={number}");
        Self { link }
    }

    async fn parse(&self, client: &reqwest::Client) -> anyhow::Result<Option<Html>> {
        let response = client.get(&self.link).send().await?;
        if response.status() != 200 {
            log!("Bad status during parsing = {}", response.status());
            return Ok(None);
        }
        let text = response.text().await?;
        let html = Html::parse_document(&text);
        Ok(Some(html))
    }
}
