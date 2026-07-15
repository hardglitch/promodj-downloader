mod helpers;
mod rows;

use crate::data::consts::{FORMS, GENRES};
use crate::data::dictionary;
use crate::db::dbcore::Database;
use eframe::{App, Frame};
use egui::{Pos2, TextureHandle, Ui};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;
use crate::data::dictionary::Lang;
use crate::logic::Command;

#[derive(Clone)]
pub struct MyApp {
    last_download_days: u64,

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
    pub lang: dictionary::Lang,
    save_to: PathBuf,
    save_tx: Option<TextureHandle>,
    last_download: usize,

    // Wallets
    qr_btc: Option<TextureHandle>,
    qr_eth: Option<TextureHandle>,
    show_qr: bool,
    qr_pos: Pos2,

    // Progress bar/Messages
    message: Option<String>,
    progress: f32,
    show_progress: bool,
    current: usize,
    total: usize,

    // Download
    dl_button_name: fn(Lang) -> &'static str,
    dl_started: bool,
    dl_paused: bool,
    play_tx: Option<TextureHandle>,
    pause_tx: Option<TextureHandle>,

    // System
    pub db: Option<Database>,
    pub common_tx: Arc<RwLock<UnboundedSender<Command>>>,
    pub common_rx: Arc<RwLock<UnboundedReceiver<Command>>>,
    pub control_tx: Arc<RwLock<UnboundedSender<Command>>>,
    pub control_rx: Arc<RwLock<UnboundedReceiver<Command>>>,
    pub client: reqwest::Client,
}
impl Default for MyApp {
    fn default() -> Self {
        let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel::<Command>();
        let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel::<Command>();

        Self {
            last_download_days: 0,

            genre: GENRES[231].0, // Techno
            form: FORMS[0],       // mixes
            quantity: 1,

            file_history: true,
            overwrite_files: true,
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

            message: None,
            progress: 0.0,
            show_progress: false,
            current: 0,
            total: 0,

            dl_button_name: dictionary::inscriptions::download,
            dl_started: false,
            dl_paused: false,
            play_tx: None,
            pause_tx: None,

            db: None,
            common_tx: Arc::new(RwLock::new(tx1)),
            common_rx: Arc::new(RwLock::new(rx1)),
            control_tx: Arc::new(RwLock::new(tx2)),
            control_rx: Arc::new(RwLock::new(rx2)),
            client: reqwest::Client::new(),
        }
    }
}
impl App for MyApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {
        if let Some(window) = frame.winit_window() {
            window.set_title(&self.window_title());
        }

        egui::CentralPanel::default().show(ui, |ui| {
            self.main_row(ui);
            ui.add_space(10.);
            self.toggles_row(ui);
            ui.add_space(20.);
            self.save_file_row(ui);
            ui.add_space(20.);
            self.progress_bar_row(ui);
            self.buttons(ui);
            self.bottom_row(ui);

            self.common_receiver(ui);
        });
    }
}
