mod rows;
mod elements;
mod helpers;

use crate::data::consts::{BASE_HEIGHT, BASE_WIDTH, FORMS, GENRES};
use crate::data::dictionary;
use crate::data::dictionary::Lang;
use crate::db::dbcore::Database;
use crate::logic::proxy::ProxyType;
use crate::logic::Command;
use eframe::{App, Frame};
use egui::{Pos2, Rect, TextureHandle, Ui, Vec2, ViewportCommand};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;
use crate::ui::helpers::button::ButtonState;

#[derive(Clone)]
pub struct MyApp {
    last_download_days: u64,

    // Dropdowns
    pub genre: &'static str,
    pub form: &'static str,
    pub quantity: String,

    // Toggle values
    pub file_history: bool,
    pub overwrite_files: bool,
    pub period: bool,
    pub lossless: bool,

    use_exclusion_filter: bool,
    xf_rect: Option<Rect>,
    xf_window_pos: Pos2,
    show_xf_window: bool,
    xf_words: String,

    use_inclusion_filter: bool,
    if_rect: Option<Rect>,
    if_window_pos: Pos2,
    show_if_window: bool,
    if_words: String,

    pub use_proxy: bool,
    show_proxy_window: bool,
    proxy_window_pos: Pos2,
    pub proxy_type: ProxyType,
    pub proxy_host: String,
    pub proxy_port: String,
    pub proxy_login: Arc<RwLock<String>>,
    pub proxy_password: Arc<RwLock<String>>,
    proxy_rect: Option<Rect>,

    settings_tx: Option<TextureHandle>,

    // Options
    pub lang: Lang,
    save_to: PathBuf,
    save_tx: Option<TextureHandle>,
    last_download: u64,

    // Wallets
    qr_btc: Option<TextureHandle>,
    qr_eth: Option<TextureHandle>,
    show_qr: bool,
    qr_pos: Pos2,
    qr_rect: Option<Rect>,

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
    pub db: Option<Arc<Database>>,
    pub common_tx: Arc<RwLock<UnboundedSender<Command>>>,
    pub common_rx: Arc<RwLock<UnboundedReceiver<Command>>>,
    pub control_tx: Arc<RwLock<UnboundedSender<Command>>>,
    pub control_rx: Arc<RwLock<UnboundedReceiver<Command>>>,
    pub client: Arc<RwLock<reqwest::Client>>,

    ui_scale: f32,
    loupe_tx: Option<TextureHandle>,
    loupe_rect: Option<Rect>,
    loupe_window_pos: Pos2,
    show_loupe_window: bool,

    buttons: HashMap<egui::Id, ButtonState>,
}
impl Default for MyApp {
    fn default() -> Self {
        let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel::<Command>();
        let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel::<Command>();

        Self {
            last_download_days: 0,

            genre: GENRES[231].0, // Techno
            form: FORMS[0],       // mixes
            quantity: "1".to_owned(),

            file_history: true,
            overwrite_files: true,
            period: true,
            lossless: true,

            use_exclusion_filter: false,
            xf_rect: None,
            xf_window_pos: Default::default(),
            show_xf_window: false,
            xf_words: String::new(),

            use_inclusion_filter: false,
            if_rect: None,
            if_window_pos: Default::default(),
            show_if_window: false,
            if_words: String::new(),

            use_proxy: false,
            show_proxy_window: false,
            proxy_window_pos: Default::default(),
            proxy_type: ProxyType::default(),
            proxy_host: "127.0.0.1".to_owned(),
            proxy_port: "8080".to_owned(),
            proxy_login: Arc::new(RwLock::new(String::new())),
            proxy_password: Arc::new(RwLock::new(String::new())),
            proxy_rect: None,
            settings_tx: None,

            lang: Lang::default(),
            save_to: PathBuf::from("Downloaded music"),
            save_tx: None,
            last_download: 0,

            qr_btc: None,
            qr_eth: None,
            show_qr: false,
            qr_pos: Default::default(),
            qr_rect: None,

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
            client: Arc::new(RwLock::new(reqwest::Client::new())),

            ui_scale: 1.0,
            loupe_tx: None,
            loupe_rect: None,
            loupe_window_pos: Default::default(),
            show_loupe_window: false,

            buttons: Default::default(),
        }
    }
}
impl App for MyApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {
        if let Some(window) = frame.winit_window() {
            window.set_title(&self.window_title());
            ui.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(BASE_WIDTH, BASE_HEIGHT)))
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.set_zoom_factor(self.ui_scale);

            self.main_row(ui);
            ui.add_space(10.);

            self.toggles_row(ui);
            ui.add_space(20.);

            self.save_file_row(ui);
            ui.add_space(20.);

            self.progress_bar_row(ui);
            self.buttons(ui);
            self.bottom_row(ui);

            self.common_receiver();
        });
    }
}
