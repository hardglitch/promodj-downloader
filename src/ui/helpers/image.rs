use eframe::CreationContext;
use eframe::epaint::ColorImage;
use eframe::epaint::textures::TextureOptions;
use image::{load_from_memory, GenericImageView, ImageBuffer, ImageError, ImageResult, Rgba};
use crate::ui::MyApp;

#[allow(dead_code)]
enum Color {
    LightGray,
    White,
    Custom(u8, u8, u8),
}

impl MyApp {
    pub(crate) fn load_textures(&mut self, ctx: &CreationContext) -> ImageResult<()> {
        let img = include_bytes!("../../../assets/qr_bitcoin.png");
        let color_image = Self::process_image(img, Color::White)?;
        let th = ctx.egui_ctx.load_texture("qr_btc", color_image, TextureOptions::default());
        self.qr_btc = Some(th);

        let img = include_bytes!("../../../assets/qr_ethereum.png");
        let color_image = Self::process_image(img, Color::White)?;
        let th = ctx.egui_ctx.load_texture("qr_eth", color_image, TextureOptions::default());
        self.qr_eth = Some(th);

        let img = include_bytes!("../../../assets/save.ico");
        let color_image = Self::process_image(img, Color::LightGray)?;
        let th = ctx.egui_ctx.load_texture("save_tx", color_image, TextureOptions::default());
        self.save_tx = Some(th);

        let img = include_bytes!("../../../assets/pause.ico");
        let color_image = Self::process_image(img, Color::LightGray)?;
        let th = ctx.egui_ctx.load_texture("pause_tx", color_image, TextureOptions::default());
        self.pause_tx = Some(th);

        let img = include_bytes!("../../../assets/play.ico");
        let color_image = Self::process_image(img, Color::LightGray)?;
        let th = ctx.egui_ctx.load_texture("play_tx", color_image, TextureOptions::default());
        self.play_tx = Some(th);

        let img = include_bytes!("../../../assets/gear.ico");
        let color_image = Self::process_image(img, Color::LightGray)?;
        let th = ctx.egui_ctx.load_texture("settings_tx", color_image, TextureOptions::default());
        self.settings_tx = Some(th);

        let img = include_bytes!("../../../assets/magnifying-glass.ico");
        let color_image = Self::process_image(img, Color::LightGray)?;
        let th = ctx.egui_ctx.load_texture("loupe_tx", color_image, TextureOptions::default());
        self.loupe_tx = Some(th);

        Ok(())
    }

    fn process_image(img: &[u8], color: Color) -> Result<ColorImage, ImageError> {
        let original_image = load_from_memory(img)?;

        // 1. Create a new buffer for the processed image
        let mut processed_image = ImageBuffer::new(
            original_image.width(),
            original_image.height(),
        );

        // 2. Iterate and apply conditional logic
        for y in 0..original_image.height() {
            for x in 0..original_image.width() {
                let pixel = original_image.get_pixel(x, y);
                let alpha = pixel[3]; // Get the original alpha value

                if alpha == 0 {
                    processed_image.put_pixel(x, y, pixel);
                } else {
                    let new_pixel = match color {
                        Color::LightGray => Rgba([200, 200, 200, alpha]),
                        Color::White => Rgba([255, 255, 255, alpha]),
                        Color::Custom(r, g, b) => Rgba([r, g, b, alpha]),
                    };
                    processed_image.put_pixel(x, y, new_pixel);
                }
            }
        }

        // 3. Create the ColorImage from the processed data
        let size = [processed_image.width() as usize, processed_image.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, &processed_image.into_raw());
        Ok(color_image)
    }
}