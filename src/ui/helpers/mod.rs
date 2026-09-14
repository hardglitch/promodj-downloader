mod image;
mod effects;
mod settings;
pub mod button;

use crate::data::dictionary::inscriptions;
use crate::db::dbcore::Database;
use crate::logic::Command;
use crate::ui::MyApp;
use eframe::epaint::text::{FontData, FontDefinitions};
use eframe::epaint::FontFamily;
use eframe::CreationContext;
use std::sync::Arc;
use x_log::log;
use crate::utils::tools;

impl MyApp {
    pub fn new(ctx: &CreationContext, db: Arc<Database>) -> Self {

        // Default settings of the App
        let mut app = MyApp {
            db: Some(db),
            ..Default::default()
        };

        // Apply a custom font
        let mut fonts = FontDefinitions::default();
        fonts.font_data
            .insert("font".to_owned(), Arc::new(FontData::from_static(include_bytes!("../../../assets/font.ttf"))));
        fonts.families.get_mut(&FontFamily::Proportional)
            .into_iter().for_each(|font| { font.insert(0, "font".to_owned()); });
        ctx.egui_ctx.set_fonts(fonts);

        // Try load the proxy settings
        if let Err(e) = app.load_proxy() { log!("Failed to load the proxy settings: {e}") }

        // Try load textures
        app.load_settings();
        if let Err(e) = app.load_textures(ctx) { log!("Failed to load textures: {e}") }
        app
    }

    pub(super) fn window_title(&mut self) -> String {
        let template = inscriptions::window_title(self.lang);
        template.replace('_', &self.last_download_days.to_string())
    }

    pub(super) fn common_receiver(&mut self) {
        if let Ok(mut cmd) = self.common_rx.try_write() &&
           let Ok(cmd) = cmd.try_recv()
        {
            match cmd {
                Command::Message(msg) => {
                    self.show_progress = false;
                    self.message = Some(msg);
                }
                Command::Progress(progress, cur, total) => {
                    self.show_progress = true;
                    self.progress = progress;
                    self.current = cur;
                    self.total = total;
                }
                Command::Stop => {
                    self.dl_button_name = inscriptions::download;
                    self.dl_started = false;
                    self.dl_paused = false;
                }
                Command::Started => {
                    self.dl_started = true;
                }
                Command::Success => {
                    self.last_download_days = 0;
                    self.last_download = tools::timestamp();
                    self.save_settings();
                }
                _ => {}
            }
        }
    }
}
