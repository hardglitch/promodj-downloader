mod helpers;
mod rows;

use crate::data::dictionary::Lang;
use eframe::{App, Frame};
use egui::{Pos2, TextureHandle, Ui};
use std::path::PathBuf;

pub struct MyApp {
    // Dropdowns
    genre: &'static str,
    form: &'static str,
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
    last_download: u64,

    // Wallets
    qr_btc: Option<TextureHandle>,
    qr_eth: Option<TextureHandle>,
    show_qr: bool,
    qr_pos: Pos2,
}
impl Default for MyApp {
    fn default() -> Self {
        Self {
            genre: "Techno",
            form: "mixes",
            quantity: 1,

            file_history: true,
            overwrite_files: false,
            period: true,
            lossless: true,

            lang: Lang::En,
            save_to: PathBuf::from("Downloaded music"),
            save_tx: None,
            last_download: 0,

            qr_btc: None,
            qr_eth: None,
            show_qr: false,
            qr_pos: Default::default(),
        }
    }
}
impl App for MyApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ui, |ui| {

            // --- Main Row
            self.main_row(ui);

            // --- GAP ---
            ui.add_space(10.);

            // --- Toggles Row ---
            self.toggles_row(ui);

            // --- GAP ---
            ui.add_space(20.);

            // --- Safe File Row ---
            self.save_file_row(ui);

            // --- GAP ---
            ui.add_space(20.);

            // --- Progress Bar/Errors Row ---
            self.progress_bar_row(ui);

            // --- Buttons
            self.buttons(ui);

            // --- The Bottom Row ---
            self.bottom_row(ui);
        });
    }
}
