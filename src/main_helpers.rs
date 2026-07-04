use crate::data::dictionary::{inscriptions, Lang};
use configparser::ini::Ini;
use egui::IconData;
use std::time::UNIX_EPOCH;

pub(super) fn load_embedded_icon(bytes: &[u8]) -> Result<IconData, image::ImageError> {
    let image = image::load_from_memory(bytes)?.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon_data = IconData { rgba, width, height };
    Ok(icon_data)
}

pub(super) fn window_title() -> String {
    let (days, lang) = last_download_days();
    let template = inscriptions::promodj_music_downloader_extended(lang);
    template.replace('_', &days.to_string())
}

fn last_download_days() -> (u64, Lang) {
    let mut config = Ini::new();
    if config.load("settings.ini").is_ok() &&
        let Ok(Some(last_ts)) = config.getuint("default", "LastDownload") &&
        let Ok(ts) = std::time::SystemTime::now().duration_since(UNIX_EPOCH) &&
        let Some(lng) = config.get("default", "Language") &&
        let Some(lang) = Lang::decode(&lng)
    {
        let days = ts.as_secs().saturating_sub(last_ts).saturating_div(3600 * 24);
        (days, lang)
    }
    else { (0, Lang::default()) }
}
