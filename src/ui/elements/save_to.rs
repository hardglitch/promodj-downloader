use eframe::emath::vec2;
use egui::{CursorIcon, Image, RichText, Sense, Ui};
use egui::load::SizedTexture;
use crate::data::dictionary::hints;
use crate::ui::MyApp;

impl MyApp {
    pub(crate) fn save_to(&mut self, ui: &mut Ui) {
        if let Some(tx_id) = &self.save_tx {
            let img = Image::new(SizedTexture::new(tx_id.id(), vec2(18., 18.)));
            let text = RichText::new(hints::save_to(self.lang));

            let button = ui.add(img.sense(Sense::click()));
            Self::glow_effect(ui, &button, tx_id.id());

            if button
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text(text)
                .clicked()
                &&
                let Some(path) = rfd::FileDialog::new()
                    .pick_folder()
            {
                self.save_to = path;
                self.save_settings();
            }
        }
    }
}