use std::sync::Arc;
use egui::IconData;
use x_log::log;
use crate::db::dbcore::{Database, DB_NAME};

pub fn load_embedded_icon(bytes: &[u8]) -> Result<IconData, image::ImageError> {
    let image = image::load_from_memory(bytes)?.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon_data = IconData { rgba, width, height };
    Ok(icon_data)
}

pub async fn load_db() -> Arc<Database> {
    let (tx, mut rx) = tokio::sync::oneshot::channel::<Database>();
    if let Some(db) = Database::create_or_open(DB_NAME).await {
        db.create_tables().await;
        if tx.send(db).is_err() { log!("Database: MPSC channel send failed"); }
    }
    if let Ok(db) = rx.try_recv() { Arc::new(db) }
    else {
        log!("An attempting to create or open the Database failed");
        panic!("An attempting to create or open the Database failed")
    }
}