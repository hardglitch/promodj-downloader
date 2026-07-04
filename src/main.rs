#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod data;
mod logging;
mod ui;
mod main_helpers;

use crate::logging::Log;
use crate::main_helpers::{load_embedded_icon, window_title};
use crate::ui::MyApp;
use eframe::{egui::ViewportBuilder, NativeOptions};
use std::io::Write;

fn main() {
    Log::init("log.log", 10 * 1024 * 1024 * 1024);

    let icon_bytes = include_bytes!("../assets/icon.png");
    let icon_data =
        match load_embedded_icon(icon_bytes) {
            Ok(d) => d,
            Err(e) => { log!("{e}"); return; }
        };

    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([600.0, 200.0])
            .with_resizable(false)
            .with_icon(icon_data)
        ,
        ..Default::default()
    };

    if let Err(e) =
        eframe::run_native(
            window_title().as_str(),
            options,
            Box::new(|ctx| Ok(Box::new(MyApp::new(ctx)))),
        )
    { log!("{e}"); }
}
