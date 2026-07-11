mod helpers;
mod rows;

use crate::data::consts::{FORMS, GENRES};
use crate::data::dictionary::Lang;
use crate::db::dbcore::Database;
use crate::logic::dsl::Data;
use eframe::{App, Frame};
use egui::{Pos2, TextureHandle, Ui};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct MyApp {
    // Dropdowns
    pub genre: &'static str,
    pub form: &'static str,
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

    // Progress bar/Messages
    message: Arc<RwLock<Option<String>>>,

    // System
    pub db: Option<Database>,
    pub tx1: Arc<RwLock<Sender<Data>>>,
    pub rx1: Arc<RwLock<Receiver<Data>>>,
    pub tx2: Arc<RwLock<Sender<Data>>>,
    pub rx2: Arc<RwLock<Receiver<Data>>>,
    is_canceled: bool,
    pub client: reqwest::Client,

    total_files: usize,
    downloaded_files: usize,
}
impl Default for MyApp {
    fn default() -> Self {
        let (tx1, rx1) = tokio::sync::mpsc::channel::<Data>(100);
        let (tx2, rx2) = tokio::sync::mpsc::channel::<Data>(100);

        Self {
            genre: GENRES[231].0, // Techno
            form: FORMS[0],       // mixes
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

            message: Arc::default(),

            db: None,
            tx1: Arc::new(RwLock::new(tx1)),
            rx1: Arc::new(RwLock::new(rx1)),
            tx2: Arc::new(RwLock::new(tx2)),
            rx2: Arc::new(RwLock::new(rx2)),
            is_canceled: false,
            client: reqwest::Client::new(),

            total_files: 0,
            downloaded_files: 0,
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
