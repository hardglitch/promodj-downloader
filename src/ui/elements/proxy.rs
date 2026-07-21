use std::net::IpAddr;
use eframe::emath::{vec2, Vec2};
use egui::{ComboBox, CursorIcon, Image, RichText, Sense, TextStyle, Ui, Window};
use egui::load::SizedTexture;
use strum::IntoEnumIterator;
use crate::data::dictionary::hints;
use crate::logic::proxy::ProxyType;
use crate::ui::MyApp;

impl MyApp {
    pub(crate) fn proxy_settings(&mut self, ui: &mut Ui) {
        if let Some(tx_id) = &self.settings_tx {
            let img = Image::new(SizedTexture::new(tx_id.id(), vec2(18., 18.)));
            let text = RichText::new(hints::proxy_settings(self.lang));

            let button = ui.add(img.sense(Sense::click()));
            Self::glow_effect(ui, &button, tx_id.id());
            self.proxy_rect = Some(button.rect);

            if button
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text(text)
                .clicked()
                &&
                let Some(pos) = ui.pointer_interact_pos()
            {
                self.proxy_window_pos.x = pos.x - 255.0;
                self.proxy_window_pos.y = pos.y + 10.0;
                self.show_proxy_window = !self.show_proxy_window;
            }
        }
    }

    pub(crate) fn proxy_popup(&mut self, ui: &mut Ui) {
        let resp = Window::new("proxy")
            .fixed_size(Vec2::new(260., 100.0))
            .title_bar(false)
            .resizable(false)
            .fixed_pos(self.proxy_window_pos)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing = Vec2::default();

                    // 1. Proxy type | Host | Port
                    // 1-1. Proxy type
                    ComboBox::new("proxy_type", "")
                        .selected_text(self.proxy_type.to_string().to_lowercase())
                        .width(75.)
                        .show_ui(ui, |ui| {
                            for proxy_type in ProxyType::iter() {
                                ui.selectable_value(&mut self.proxy_type, proxy_type, proxy_type.to_string());
                            }
                        });

                    ui.add_space(5.);

                    // 1-2. Host
                    let state_before = self.proxy_host.clone();
                    let widget = egui::widgets::TextEdit::singleline(&mut self.proxy_host)
                        .desired_width(110.);
                    ui.add(widget);
                    if self.proxy_host == state_before ||
                        self.proxy_host.parse::<IpAddr>().is_err()
                    {
                        self.proxy_host = state_before
                    }

                    ui.add_space(2.);
                    ui.label(":");
                    ui.add_space(2.);

                    // 1-3. Port
                    let state_before = self.proxy_port.clone();
                    let widget = egui::widgets::TextEdit::singleline(&mut self.proxy_port)
                        .desired_width(50.);
                    ui.add(widget);
                    if self.proxy_port == state_before ||
                        self.proxy_port.parse::<u16>().is_err()
                    {
                        self.proxy_port = state_before
                    }

                });

                // 2. Auth
                // 2-1. Login (disabled if not http_auth)
                ui.horizontal(|ui| {
                    if let Ok(mut buf) = self.proxy_login.try_write() {
                        let enabled = matches!(self.proxy_type, ProxyType::HttpAuth);
                        let widget = egui::widgets::TextEdit::singleline(&mut *buf)
                            .char_limit(255)
                            .desired_width(ui.available_width())
                            .hint_text(hints::login(self.lang))
                            .font(TextStyle::Heading)
                            .desired_rows(1)
                            .interactive(enabled);

                        ui.add(widget);
                    }
                });

                // 2-2. Password (disabled if not http_auth)
                ui.horizontal(|ui| {
                    if let Ok(mut buf) = self.proxy_password.try_write() {
                        let enabled = matches!(self.proxy_type, ProxyType::HttpAuth);
                        let widget = egui::widgets::TextEdit::singleline(&mut *buf)
                            .char_limit(255)
                            .desired_width(ui.available_width())
                            .hint_text(hints::password(self.lang))
                            .font(TextStyle::Heading)
                            .desired_rows(1)
                            .password(true)
                            .interactive(enabled);

                        ui.add(widget);
                    }
                });
            });

        // Hide the popup window if mouse was clicked from outside
        ui.input(|i| {
            if i.pointer.any_click() &&
                let Some(click_pos) = i.pointer.interact_pos() &&
                let Some(r) = resp
            {
                // dbg!(&r.response.rect, &self.proxy_rect, &click_pos);
                if !r.response.rect.contains(click_pos) &&
                    self.proxy_rect.is_some_and(|r| !r.contains(click_pos))
                {
                    self.show_proxy_window = false;
                    self.save_proxy();
                }
            }
        });
    }
}