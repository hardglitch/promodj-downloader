#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod data;
mod logging;
mod ui;

use eframe::{egui::{IconData, ViewportBuilder}, NativeOptions};
use std::io::Write;
use crate::logging::Log;
use crate::ui::MusicDownloaderApp;

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
            "PromoDJ Music Downloader",
            options,
            Box::new(|_cc| Ok(Box::<MusicDownloaderApp>::default())),
        )
    { log!("{e}"); }
}

fn load_embedded_icon(bytes: &[u8]) -> Result<IconData, image::ImageError> {
    let image = image::load_from_memory(bytes)?.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon_data = IconData { rgba, width, height };
    Ok(icon_data)
}
