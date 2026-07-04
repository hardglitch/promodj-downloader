mod helpers;

use crate::data::consts::{FORMS, GENRES};
use crate::data::dictionary::*;
use crate::data::dictionary::Lang;
use eframe::{App, Frame};
use egui::{Align, ComboBox, Layout, Pos2, TextureHandle, Ui};
use std::path::PathBuf;

pub struct MyApp {
    // Dropdowns
    genre: &'static str,
    form: &'static str,
    last: usize,
    quantity: u64,

    // Toggle values
    file_history: bool,
    overwrite_files: bool,
    period: bool,
    lossless: bool,

    // Options
    lang: Lang,
    save_to: PathBuf,
    save_tx: Option<TextureHandle>,

    // Wallets
    qr_btc: Option<TextureHandle>,
    qr_eth: Option<TextureHandle>,
    copy_tx: Option<TextureHandle>,
    show_qr: bool,
    qr_pos: Pos2,
}
impl Default for MyApp {
    fn default() -> Self {
        Self {
            genre: "Techno",
            form: "mixes",
            last: 0,
            quantity: 1,

            file_history: true,
            overwrite_files: false,
            period: true,
            lossless: true,

            lang: Lang::En,
            save_to: PathBuf::from("Downloaded music"),
            save_tx: None,

            qr_btc: None,
            qr_eth: None,
            copy_tx: None,
            show_qr: false,
            qr_pos: Default::default(),
        }
    }
}
impl App for MyApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ui, |ui| {

            // --- Main Row
            ui.horizontal(|ui| {
                ComboBox::new("genre", "")
                    .selected_text(&*self.genre)
                    .width(250.)
                    .show_ui(ui, |ui| {
                        for (text, _) in GENRES.iter() {
                            ui.selectable_value(&mut self.genre, text, *text);
                        }
                    });

                ComboBox::new("form", "")
                    .selected_text(&*self.form)
                    .show_ui(ui, |ui| {
                        for form in FORMS.iter() {
                            ui.selectable_value(&mut self.form, form, *form);
                        }
                    });

                ui.add(egui::DragValue::new(&mut self.last)
                           .speed(1.0)
                           .range(0.0..=1000.0)
                );
                let last =
                    if self.period { inscriptions::last_days(self.lang) }
                    else { inscriptions::last_files(self.lang) };
                ui.label(last);
            });

            // --- GAP ---
            ui.add_space(10.);

            // --- Toggles Row ---
            ui.horizontal(|ui| {
                ui.add_space(150.);
                ui.toggle_value(&mut self.file_history, inscriptions::file_history(self.lang));
                ui.toggle_value(&mut self.overwrite_files, inscriptions::overwrite_files(self.lang));
                ui.toggle_value(&mut self.period, inscriptions::period(self.lang));
                ui.toggle_value(&mut self.lossless, inscriptions::lossless(self.lang));
            });

            // --- GAP ---
            ui.add_space(10.);

            // --- Safe File Row ---
            ui.horizontal(|ui| {
                self.save_to(ui);
                if let Some(p) = self.save_to.as_path().to_str() {
                    ui.label(p);
                }
            });

            // --- Progress Bar/Errors Row ---
            ui.horizontal(|ui| {

            });

            // --- Action Buttons Row ---
            ui.horizontal(|ui| {
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    ui.label("v0.8");
                    ui.hyperlink_to("hardglitch", "https://github.com/hardglitch");
                    self.donate(ui);
                    self.donate_popup(ui);
                    self.lang_switcher(ui);
                });

                ui.with_layout(Layout::left_to_right(Align::Max), |ui| {
                    if ui.button(inscriptions::exit(self.lang)).clicked() {
                        println!("EXIT clicked.");
                    }
                    if ui.button(inscriptions::download(self.lang)).clicked() {
                        println!("DOWNLOAD initiated with settings");
                    }
                });
            });
        });
    }
}
