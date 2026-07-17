use crate::data::consts::{GENRES, LOSSLESS_COMPRESSED_FORMATS, LOSSLESS_UNCOMPRESSED_FORMATS, LOSSY_FORMATS, MAX_QUANTITY};
use crate::data::dictionary;
use crate::data::dictionary::Lang;
use crate::db::dbcore::Database;
use crate::log;
use anyhow::anyhow;
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use percent_encoding::percent_decode_str;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use crate::logic::Command;

pub struct LinkParams<'a> {
    pub form: &'a str,
    pub genre: &'a str,
    pub quantity: usize,
    pub period: bool,
    pub lang: Lang,
    pub file_history: bool,
    pub lossless: bool,
    pub client: reqwest::Client,
    pub db: Option<Arc<Database>>,
    pub common_tx: Arc<RwLock<UnboundedSender<Command>>>,
}

pub struct Link;
impl Link {
    pub async fn get_all_links<'a>(link_params: LinkParams<'a>) -> anyhow::Result<Option<Vec<String>>> {
        let searching = UiActionHandle::run(UiAction::Searching, link_params.lang, link_params.common_tx.clone());

        // 1. Get the link set
        let mut found_links: HashSet<String> = HashSet::new();
        let mut page_number = 1;

        while (found_links.len() < link_params.quantity && !link_params.period) ||
              (found_links.len() < MAX_QUANTITY && link_params.period)
        {
            // If we found nothing on this page, stop searching
            if page_number > 1 && found_links.is_empty() { break; }
			
			// This is for safe scanning
			if page_number > 1 {
				tokio::time::sleep(Duration::from_millis(500)).await;
			}

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
                        if let Ok(tx) = link_params.common_tx.try_read() {
                            tx.send(Command::Message(msg.to_owned()))?;
                        }
                        return Ok(None);
                    }
                };

            if !found_links_on_page.is_empty() {
                found_links.extend(found_links_on_page);
            }
            // If we found nothing on this page, stop searching
            else { break; }

            page_number += 1;
        }

        drop(searching);
        let analysis = UiActionHandle::run(UiAction::Analysis, link_params.lang, link_params.common_tx.clone());

        // 2. Remove duplicates
        //    Convert {"1.wav", "1.flac", "2.flac", "2.wav"} to {'1.flac', '2.flac'}
        let mut unique_links: HashMap<&str, &str> = HashMap::new();
        for link in &found_links {
            if let Some((name, ext)) = link.rsplit_once('.') {
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
            if let Ok(tx) = link_params.common_tx.try_read() {
                tx.send(Command::Message(msg.to_owned()))?;
            }
            return Ok(None);
        }

        // 3. Check found links in the history
        let mut found_links = Vec::<String>::new();
        if link_params.file_history &&
           let Some(db) = link_params.db &&
           let Some(links) = db.filter_by_history(unique_links).await
        {
            found_links = links;
        }

        // 4. Truncate found links
        if link_params.period { found_links.truncate(MAX_QUANTITY) }
        else { found_links.truncate(link_params.quantity) };

        if found_links.is_empty() {
            let msg = dictionary::errors::no_links_to_download(link_params.lang);
            if let Ok(tx) = link_params.common_tx.try_read() {
                tx.send(Command::Message(msg.to_owned()))?;
            }
            return Ok(None);
        }

        drop(analysis);
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

                let is_source =
                    if cfg!(feature = "test") { true }
                    else { href.find("/source/").is_some() };

                if format_matches && is_source {
                    let decoded_href = percent_decode_str(href).decode_utf8()?.to_lowercase();
                    links.insert(decoded_href);
                }
            }
        }
        Ok(links)
    }
}

#[derive(Copy, Clone)]
enum UiAction {
    Searching,
    Analysis,
}
struct UiActionHandle {
    handle: JoinHandle<()>,
}
impl UiActionHandle {
    fn run(action: UiAction, lang: Lang, common_tx: Arc<RwLock<UnboundedSender<Command>>>) -> Self {
        let handle = Self::handle(action, lang, common_tx);
        Self { handle }
    }
    fn handle(action: UiAction, lang: Lang, common_tx: Arc<RwLock<UnboundedSender<Command>>>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut i = 1;
            let action_ = match action {
                UiAction::Searching => dictionary::ui_messages::searching(lang),
                UiAction::Analysis => dictionary::ui_messages::analysis(lang),
            };

            loop {
                let dots = ".".repeat(i % 5);
                let msg = format!("{dots}{action_}{dots}");
                if let Ok(tx) = common_tx.try_read() {
                    let _ = tx.send(Command::Message(msg));
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
                i += 1;
            }
        })
    }
}
impl Drop for UiActionHandle {
    fn drop(&mut self) {
        self.handle.abort();
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
