use eframe::emath::{vec2, Vec2};
use egui::{CursorIcon, Image, RichText, Sense, Ui, Window};
use egui::load::SizedTexture;
use crate::data::dictionary::hints;
use crate::ui::MyApp;

impl MyApp {
    pub(crate) fn inclusion_filter_settings(&mut self, ui: &mut Ui) {
        if let Some(tx_id) = &self.settings_tx {
            let img = Image::new(SizedTexture::new(tx_id.id(), vec2(18., 18.)));
            let text = RichText::new(hints::filter_settings(self.lang));

            let button = ui.add(img.sense(Sense::click()));
            Self::glow_effect(ui, &button, tx_id.id());
            self.if_rect = Some(button.rect);

            if button
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text(text)
                .clicked()
                &&
                let Some(pos) = ui.pointer_interact_pos()
            {
                self.if_window_pos.x = pos.x - 250.0;
                self.if_window_pos.y = pos.y + 10.0;
                self.show_if_window = !self.show_if_window;
            }
        }
    }

    pub(crate) fn inclusion_filter_popup(&mut self, ui: &mut Ui) {
        let resp = Window::new("if_settings")
            .fixed_size(Vec2::new(300., 100.0))
            .title_bar(false)
            .resizable(false)
            .fixed_pos(self.if_window_pos)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::widgets::TextEdit::singleline(&mut self.if_words)
                            .char_limit(10000)
                            .hint_text(hints::filter_example())
                    );
                });
            });

        // Hide the popup window if mouse was clicked from outside
        ui.input(|i| {
            if i.pointer.any_click() &&
                let Some(click_pos) = i.pointer.interact_pos() &&
                let Some(r) = resp
            {
                // dbg!(&r.response.rect, &self.loupe_rect, &click_pos);
                if !r.response.rect.contains(click_pos) &&
                    self.if_rect.is_some_and(|r| !r.contains(click_pos))
                {
                    self.show_if_window = false;
                    self.save_settings();
                }
            }
        });
    }
}