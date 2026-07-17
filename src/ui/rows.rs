use crate::data::consts::{FORMS, GENRES, VERSION};
use crate::data::dictionary::{hints, inscriptions};
use crate::ui::MyApp;
use eframe::emath::Align;
use egui::{ComboBox, CursorIcon, Layout, Ui};

impl MyApp {
    pub(super) fn main_row(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {

            // Genre
            let state_before = self.genre;
            ComboBox::new("genre", "")
                .selected_text(self.genre)
                .width(250.)
                .show_ui(ui, |ui| {
                    for (text, _) in GENRES.into_iter() {
                        ui.selectable_value(&mut self.genre, text, text);
                    }
                }).response.on_hover_text(hints::genre(self.lang));
            if self.genre != state_before { self.save_settings(); }

            // Form
            let state_before = self.form;
            ComboBox::new("form", "")
                .selected_text(self.form)
                .show_ui(ui, |ui| {
                    for form in FORMS.into_iter() {
                        ui.selectable_value(&mut self.form, form, form);
                    }
                });
            if self.form != state_before { self.save_settings(); }

            // Quantity
            let state_before = self.quantity.clone();
            ui.add(
                egui::widgets::TextEdit::singleline(&mut self.quantity)
                    .desired_width(40.)
            )
                .on_hover_text(hints::quantity(self.lang));
            if self.quantity != state_before &&
               self.quantity.parse::<u16>().is_ok_and(|n| n <= 1000)
            { self.save_settings(); }
            else { self.quantity = state_before }

            let last =
                if self.period { inscriptions::last_days(self.lang) }
                else { inscriptions::last_files(self.lang) };
            ui.label(last);
        });
    }

    pub(super) fn toggles_row(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(10.);

            ui.vertical(|ui| {
                ui.set_width(60.);
                ui.with_layout(Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                        if ui
                        .toggle_value(&mut self.file_history, inscriptions::file_history(self.lang))
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(hints::file_history(self.lang))
                        .clicked()
                    {
                        self.save_settings();
                    }
                });
            });

            ui.vertical(|ui| {
                ui.set_width(155.);
                ui.with_layout(Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                    if self.file_history { self.overwrite_files = true; }
                    if ui
                        .toggle_value(&mut self.overwrite_files, inscriptions::overwrite_files(self.lang))
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(hints::overwrite_files(self.lang))
                        .clicked()
                            &&
                        !self.file_history
                    {
                        self.save_settings();
                    }
                });
            });

            ui.vertical(|ui| {
                ui.set_width(55.);
                ui.with_layout(Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                    if ui
                        .toggle_value(&mut self.period, inscriptions::period(self.lang))
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(hints::period(self.lang))
                        .clicked()
                    {
                        self.save_settings();
                    }
                });
            });

            ui.vertical(|ui| {
                ui.set_width(80.);
                ui.with_layout(Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                    if ui
                        .toggle_value(&mut self.lossless, inscriptions::lossless(self.lang))
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(hints::lossless(self.lang))
                        .clicked()
                    {
                        self.save_settings();
                    }
                });
            });

            ui.vertical(|ui| {
                ui.set_width(50.);
                ui.with_layout(Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                    if ui
                        .toggle_value(&mut self.use_proxy, "Proxy")
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(hints::proxy(self.lang))
                        .clicked()
                    {
                        self.save_settings();
                    }
                });
            });

            ui.vertical(|ui| {
                self.proxy_settings(ui);
            });
        });

        if self.show_proxy_window { self.proxy_popup(ui); }
    }

    pub(super) fn save_file_row(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.save_to(ui);
            if let Some(p) = self.save_to.as_path().to_str() {
                ui.label(p);
            }
        });
    }

    pub(super) fn progress_bar_row(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                if self.show_progress {
                    self.progress_bar(ui);
                }
                else if let Some(msg) = &self.message {
                   ui.label(msg);
                }
                ui.request_repaint();
            });
        });
    }

    pub(super) fn buttons(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
            let legend = self.dl_button_name;
            let button_text = egui::RichText::new(legend(self.lang)).size(24.0);
            let button = egui::Button::new(button_text)
                .min_size(egui::vec2(170., 30.));

            if ui.add(button)
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                self.download();
            }
            if self.dl_started { self.pause(ui); }
        });
    }

    pub(super) fn bottom_row(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    ui.label(VERSION);
                    ui
                        .hyperlink_to("hardglitch", "https://github.com/hardglitch")
                        .on_hover_text("https://github.com/hardglitch");
                    self.donate(ui);
                    if self.show_qr { self.donate_popup(ui); }
                    self.lang_switcher(ui);
                });
            });
        });
    }
}