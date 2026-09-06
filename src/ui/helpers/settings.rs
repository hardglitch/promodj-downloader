use std::path::PathBuf;
use std::str::FromStr;
use std::time::UNIX_EPOCH;
use configparser::ini::Ini;
use log::log;
use crate::data::consts::{FORMS, GENRES};
use crate::data::dictionary::Lang;
use crate::ui::MyApp;

impl MyApp {
    pub(crate) fn save_settings(&self) {
        let mut config = Ini::new();

        config.set("default", "LastDownload", Some(self.last_download.to_string()));
        config.set("default", "Language", Some(self.lang.encode().to_owned()));

        let s = self.save_to.clone().into_os_string().into_string().ok();
        config.set("default", "DownloadDirectory", s);

        config.set("default", "Genre", Some(self.genre.to_owned()));
        config.set("default", "Form", Some(self.form.to_owned()));
        config.set("default", "Lossless", Some(self.lossless.to_string()));
        config.set("default", "Period", Some(self.period.to_string()));
        config.set("default", "OverwriteFiles", Some(self.overwrite_files.to_string()));
        config.set("default", "FileHistory", Some(self.file_history.to_string()));

        if let Ok(quantity) = self.quantity.parse::<u16>() {
            config.set("default", "Quantity", Some(quantity.to_string()));
        }

        config.set("default", "UseXF", Some(self.use_exclusion_filter.to_string()));
        config.set("default", "XFWords", Some(self.xf_words.clone()));

        config.set("default", "UseIF", Some(self.use_inclusion_filter.to_string()));
        config.set("default", "IFWords", Some(self.if_words.clone()));

        config.set("default", "Proxy", Some(self.use_proxy.to_string()));
        config.set("default", "Scale", Some(self.ui_scale.to_string()));

        if let Err(e) = config.write("settings.ini") { log!("Config: {e}"); }
    }

    pub(crate) fn load_settings(&mut self) {
        let mut config = Ini::new();
        if config.load("settings.ini").is_ok() {
            if let Ok(Some(last_ts)) = config.getuint("default", "LastDownload") {
                self.last_download = last_ts;

                if last_ts > 0 &&
                    let Ok(ts) = std::time::SystemTime::now().duration_since(UNIX_EPOCH)
                {
                    let days = ts.as_secs().saturating_sub(last_ts).saturating_div(3600 * 24);
                    self.last_download_days = days;
                }

            }
            if let Some(lng) = config.get("default", "Language") &&
                let Some(lang) = Lang::decode(&lng)
            {
                self.lang = lang;
            }
            if let Some(dir) = config.get("default", "DownloadDirectory") &&
                let Ok(path) = PathBuf::from_str(&dir)
            {
                self.save_to = path;
            }
            if let Some(genre) = config.get("default", "Genre") &&
                let Some(g) = GENRES.iter().find_map(|(name, _)| {
                    if name == &genre { Some(name) } else { None }
                })
            {
                self.genre = g;
            }
            if let Some(form) = config.get("default", "Form") &&
                let Some(form_) = FORMS.iter().find(|&f| f == &form)
            {
                self.form = form_;
            }
            if let Ok(Some(ls)) = config.getbool("default", "Lossless") {
                self.lossless = ls;
            }
            if let Ok(Some(p)) = config.getbool("default", "Period") {
                self.period = p;
            }
            if let Ok(Some(of)) = config.getbool("default", "OverwriteFiles") {
                self.overwrite_files = of;
            }
            if let Ok(Some(fh)) = config.getbool("default", "FileHistory") {
                self.file_history = fh;
            }
            if let Ok(Some(q)) = config.getuint("default", "Quantity") {
                self.quantity = (q as usize).to_string();
            }
            if let Ok(Some(xf)) = config.getbool("default", "UseXF") {
                self.use_exclusion_filter = xf;
            }
            if let Some(xf_words) = config.get("default", "XFWords") {
                self.xf_words = xf_words;
            }
            if let Ok(Some(in_f)) = config.getbool("default", "UseIF") {
                self.use_inclusion_filter = in_f;
            }
            if let Some(if_words) = config.get("default", "IFWords") {
                self.if_words = if_words;
            }
            if let Ok(Some(p)) = config.getbool("default", "Proxy") {
                self.use_proxy = p;
            }
            if let Ok(Some(s)) = config.getfloat("default", "Scale") {
                self.ui_scale = s as f32;
            }
        }
    }
}