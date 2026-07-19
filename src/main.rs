#![feature(async_fn_traits)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod data;
mod utils;
mod ui;
mod logic;
mod db;
#[cfg(test)]
mod test_server;
#[cfg(test)]
mod db_converter;

use ui::MyApp;
use eframe::{egui::ViewportBuilder, NativeOptions};
use std::io::Write;
use utils::main_helpers::{load_embedded_icon, load_db};
use utils::logging::Log;
use crate::data::consts::{BASE_HEIGHT, BASE_WIDTH};

#[tokio::main]
async fn main() {
    Log::init("log.log", 10 * 1024 * 1024 * 1024);

    let icon_bytes = include_bytes!("../assets/icon.ico");
    let icon_data =
        match load_embedded_icon(icon_bytes) {
            Ok(d) => d,
            Err(e) => { log!("{e}"); return; }
        };

    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([BASE_WIDTH, BASE_HEIGHT])
            .with_resizable(false)
            .with_icon(icon_data)
        ,
        ..Default::default()
    };

    let db = load_db().await;

    if let Err(e) =
        eframe::run_native(
            "",
            options,
            Box::new(|ctx| Ok(Box::new(MyApp::new(ctx, db)))),
        )
    { log!("{e}"); }
}