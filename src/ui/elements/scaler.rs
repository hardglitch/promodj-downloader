use eframe::emath::{vec2, Vec2};
use egui::{CursorIcon, Image, RichText, Sense, Ui, Window};
use egui::load::SizedTexture;
use crate::data::dictionary::hints;
use crate::ui::MyApp;

impl MyApp {
    pub(crate) fn ui_scale(&mut self, ui: &mut Ui) {
        if let Some(tx_id) = &self.loupe_tx {
            let img = Image::new(SizedTexture::new(tx_id.id(), vec2(18., 18.)));
            let text = RichText::new(hints::ui_scale(self.lang));

            let button = ui.add(img.sense(Sense::click()));
            Self::glow_effect(ui, &button, tx_id.id());
            self.loupe_rect = Some(button.rect);

            if button
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text(text)
                .clicked()
                &&
                let Some(pos) = ui.pointer_interact_pos()
            {
                self.loupe_window_pos.x = pos.x - 37.0;
                self.loupe_window_pos.y = pos.y - 40.0;
                self.show_loupe_window = !self.show_loupe_window;
            }
        }
    }
    pub(crate) fn ui_scale_popup(&mut self, ui: &mut Ui) {
        let resp = Window::new("loupe")
            .fixed_size(Vec2::new(100., 25.0))
            .title_bar(false)
            .resizable(false)
            .fixed_pos(self.loupe_window_pos)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let state_before = self.ui_scale;
                    if ui.add(egui::widgets::Button::new("-")).clicked() && self.ui_scale > 1. {
                        self.ui_scale -= 0.25;
                    };
                    ui.label(RichText::new(format!("{:.2}", self.ui_scale)));
                    if ui.add(egui::widgets::Button::new("+")).clicked() && self.ui_scale < 4. {
                        self.ui_scale += 0.25;
                    };
                    if self.ui_scale != state_before {
                        self.save_settings();
                    }
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
                    self.loupe_rect.is_some_and(|r| !r.contains(click_pos))
                {
                    self.show_loupe_window = false;
                    self.save_settings();
                }
            }
        });
    }
}