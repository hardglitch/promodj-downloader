use egui::IconData;

pub(crate) fn load_embedded_icon(bytes: &[u8]) -> Result<IconData, image::ImageError> {
    let image = image::load_from_memory(bytes)?.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon_data = IconData { rgba, width, height };
    Ok(icon_data)
}
