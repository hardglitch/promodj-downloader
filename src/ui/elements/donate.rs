use eframe::emath::{vec2, Align, Vec2};
use eframe::epaint::{Color32, TextureId};
use egui::{CursorIcon, Image, Label, Layout, RichText, Sense, Ui, Window};
use egui::load::SizedTexture;
use crate::data::dictionary::hints;
use crate::ui::MyApp;

impl MyApp {
    pub(crate) fn donate(&mut self, ui: &mut Ui) {
        let donate_text = RichText::new("donate");
        let donate_btn = Label::new(donate_text);
        let donate = ui
            .add(donate_btn.sense(Sense::click()))
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(hints::donate(self.lang));

        self.qr_rect = Some(donate.rect);

        if donate.clicked() &&
            let Some(pos) = ui.pointer_interact_pos()
        {
            self.qr_pos.x = pos.x + 10.0;
            self.qr_pos.y = pos.y - 190.0;
            self.show_qr = !self.show_qr;
        }
    }

    pub(crate) fn donate_popup(&mut self, ui: &mut Ui) {
        let resp = Window::new("donate")
            .fixed_size(Vec2::new(320., 160.0))
            .title_bar(false)
            .resizable(false)
            .fixed_pos(self.qr_pos)
            .show(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    let th_btc_id = if let Some(t) = &self.qr_btc { t.id() } else { return };
                    let th_eth_id = if let Some(t) = &self.qr_eth { t.id() } else { return };

                    ui.vertical(|ui| {
                        let img = Image::new(SizedTexture::new(th_btc_id, vec2(150.0, 150.0)));
                        let wallet = "bc1qfyt84p8t85pg6597882cr7p04ank2263t7asa6";
                        self.copy_to_clipboard(img, th_btc_id, wallet, ui);

                        ui.horizontal(|ui| {
                            ui.add_space(60.);
                            ui.colored_label(Color32::ORANGE, "bitcoin");
                        });
                    });

                    ui.vertical(|ui| {
                        let img = Image::new(SizedTexture::new(th_eth_id, vec2(150.0, 150.0)));
                        let wallet = "0x1991F455084DfF493AC13D0473d92b47A80403F9";
                        self.copy_to_clipboard(img, th_eth_id, wallet, ui);

                        ui.horizontal(|ui| {
                            ui.add_space(50.);
                            ui.colored_label(Color32::from_rgb(157,167,218), "ethereum");
                        });
                    });
                });
            });

        // Hide the popup window if mouse was clicked from outside
        ui.input(|i| {
            if i.pointer.any_click() &&
                let Some(click_pos) = i.pointer.interact_pos() &&
                let Some(r) = resp
            {
                // dbg!(&r.response.rect, &self.qr_rect, &click_pos);
                if !r.response.rect.contains(click_pos) &&
                    self.qr_rect.is_some_and(|r| !r.contains(click_pos))
                {
                    self.show_qr = false;
                }
            }
        });
    }

    fn copy_to_clipboard(&mut self, img: Image, texture_id: TextureId, text: &str, ui: &mut Ui) {
        let button = ui.add(img.sense(Sense::click()));
        Self::glow_effect(ui, &button, texture_id);
        self.flash_effect(ui, &button, texture_id);

        let id = button.id;
        if button
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(hints::copy(self.lang))
            .clicked()
        {
            self.enable_flash(id);
            ui.copy_text(text.to_string());
        };
    }
}