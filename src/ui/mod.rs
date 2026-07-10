mod helpers;
mod rows;

use crate::data::dictionary::Lang;
use eframe::{App, Frame};
use egui::{Pos2, TextureHandle, Ui};
use std::path::PathBuf;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::db::dbcore::Database;
use crate::logic::dsl::{Command, Data};

pub struct MyApp<'a> {
    // Dropdowns
    pub genre: &'a str,
    pub form: &'a str,
    pub quantity: usize,

    // Toggle values
    pub file_history: bool,
    pub overwrite_files: bool,
    pub period: bool,
    pub lossless: bool,

    // Options
    pub lang: Lang,
    save_to: PathBuf,
    save_tx: Option<TextureHandle>,
    last_download: usize,

    // Wallets
    qr_btc: Option<TextureHandle>,
    qr_eth: Option<TextureHandle>,
    show_qr: bool,
    qr_pos: Pos2,

    // System
    pub db: Option<Database>,
    pub tx: Option<Sender<Data>>,
    pub rx: Option<Receiver<Data>>,
    is_canceled: bool,
    pub client: reqwest::Client,

    total_files: usize,
    downloaded_files: usize,
}
impl<'a> Default for MyApp<'a> {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<Data>(100);
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

            db: None,
            tx: Some(tx),
            rx: Some(rx),
            is_canceled: false,
            client: reqwest::Client::new(),

            total_files: 0,
            downloaded_files: 0,
        }
    }
}
impl<'a> App for MyApp<'a> {
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
