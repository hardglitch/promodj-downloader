use crate::data::consts::{FORMS, GENRES};
use crate::data::dictionary;
use crate::data::dictionary::{hints, inscriptions, Lang};
use crate::db::dbcore::{Database, DB_NAME};
use crate::log;
use crate::logic::file::download_files;
use crate::logic::search::{Link, LinkParams};
use crate::ui::MyApp;
use configparser::ini::Ini;
use eframe::emath::{vec2, Align, Rect};
use eframe::epaint::text::{FontData, FontDefinitions};
use eframe::epaint::textures::TextureOptions;
use eframe::epaint::{Color32, ColorImage, FontFamily};
use eframe::CreationContext;
use egui::load::SizedTexture;
use egui::{CursorIcon, FontId, Image, Label, Layout, Pos2, RichText, Sense, Ui, Vec2, Window};
use image::{GenericImageView, ImageBuffer};
use image::{ImageError, ImageResult, Rgba};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use crate::logic::Command;

impl MyApp {
    pub fn new(ctx: &CreationContext) -> Self {
        let mut fonts = FontDefinitions::default();
        fonts.font_data
            .insert(
                "font".to_owned(),
                Arc::new(FontData::from_static(include_bytes!("../../assets/font.ttf")))
            );
        fonts.families.get_mut(&FontFamily::Proportional).unwrap()
            .insert(0, "font".to_owned());

        ctx.egui_ctx.set_fonts(fonts);

        let mut app = Self::default();

        // Try open history.db
        let (tx, mut rx) = tokio::sync::oneshot::channel::<Database>();
        tokio::spawn(async move {
            if let Some(db) = Database::create_or_open(DB_NAME).await {
                db.create_history_db().await;
                if tx.send(db).is_err() { log!("Database: MPSC channel send failed"); }
            }
        });
        app.db = rx.try_recv().ok();

        app.load_settings();
        if let Err(e) = app.load_textures(ctx) { log!("Load textures: {e}") }
        app
    }

    pub(super) fn save_settings(&mut self) {
        let mut config = Ini::new();

        config.set("default", "LastDownload", Some(self.last_download.to_string()));
        config.set("default", "Language", Some(self.lang.to_string()));

        let s = self.save_to.clone().into_string().ok();
        config.set("default", "DownloadDirectory", s);

        config.set("default", "Genre", Some(self.genre.to_owned()));
        config.set("default", "Form", Some(self.form.to_owned()));
        config.set("default", "Lossless", Some(self.lossless.to_string()));
        config.set("default", "Period", Some(self.period.to_string()));
        config.set("default", "OverwriteFiles", Some(self.overwrite_files.to_string()));
        config.set("default", "FileHistory", Some(self.file_history.to_string()));
        config.set("default", "Quantity", Some(self.quantity.to_string()));

        if let Err(e) = config.write("settings.ini") { log!("Config: {e}"); }
    }
    fn load_settings(&mut self) {
        let mut config = Ini::new();
        if config.load("settings.ini").is_ok() {

            // Last download
            if let Ok(Some(ts)) = config.getuint("default", "LastDownload") {
                self.last_download = ts as usize;
            }

            // Language
            if let Some(lng) = config.get("default", "Language") &&
               let Some(lang) = Lang::decode(&lng)
            {
                self.lang = lang;
            }

            // Download directory
            if let Some(dir) = config.get("default", "DownloadDirectory") &&
               let Ok(path) = PathBuf::from_str(&dir)
            {
                self.save_to = path;
            }

            // Genre
            if let Some(genre) = config.get("default", "Genre") &&
               let Some(g) = GENRES.iter().find_map(|(name, _)| {
                   if name == &genre { Some(name) } else { None }
               })
            {
                self.genre = g;
            }

            // Form
            if let Some(form) = config.get("default", "Form") &&
               let Some(form_) = FORMS.iter().find(|&f| f == &form)
            {
                self.form = form_;
            }

            // Lossless
            if let Ok(Some(ls)) = config.getbool("default", "Lossless") {
                self.lossless = ls;
            }

            // Period
            if let Ok(Some(p)) = config.getbool("default", "Period") {
                self.period = p;
            }

            // Overwrite files
            if let Ok(Some(of)) = config.getbool("default", "OverwriteFiles") {
                self.overwrite_files = of;
            }

            // File history
            if let Ok(Some(fh)) = config.getbool("default", "FileHistory") {
                self.file_history = fh;
            }

            // Quantity
            if let Ok(Some(q)) = config.getuint("default", "Quantity") {
                self.quantity = q as usize;
            }
        }
    }

    fn load_textures(&mut self, ctx: &CreationContext) -> ImageResult<()> {
        let img = include_bytes!("../../assets/qr_bitcoin.png");
        let image = image::load_from_memory(img)?.into_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, &image.into_raw());
        let th = ctx.egui_ctx.load_texture("qr_btc", color_image, TextureOptions::default());
        self.qr_btc = Some(th);

        let img = include_bytes!("../../assets/qr_ethereum.png");
        let image = image::load_from_memory(img)?.into_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, &image.into_raw());
        let th = ctx.egui_ctx.load_texture("qr_eth", color_image, TextureOptions::default());
        self.qr_eth = Some(th);

        let img = include_bytes!("../../assets/save.ico");
        let image = image::load_from_memory(img)?.into_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, &image.into_raw());
        let th = ctx.egui_ctx.load_texture("save_tx", color_image, TextureOptions::default());
        self.save_tx = Some(th);

        Ok(())
    }

    // Paint any image to light-gray color
    fn process_image(img: &[u8]) -> Result<ColorImage, ImageError> {
        let original_image = image::load_from_memory(img)?;

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
                    let new_pixel = Rgba([200, 200, 200, alpha]);
                    processed_image.put_pixel(x, y, new_pixel);
                }
            }
        }

        // 3. Create the ColorImage from the processed data
        let size = [processed_image.width() as usize, processed_image.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, &processed_image.into_raw());
        Ok(color_image)
    }

    pub(super) fn lang_switcher(&mut self, ui: &mut Ui) {
        let lang_text = RichText::new(self.lang.to_string().to_lowercase());
        let lang_btn = Label::new(lang_text);
        if ui
            .add(lang_btn.sense(Sense::click()))
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(hints::switch_language(self.lang))
            .clicked()
        {
            self.lang = match self.lang {
                Lang::En => Lang::Ru,
                Lang::Ru => Lang::Uk,
                Lang::Uk => Lang::En,
            };
            self.save_settings();
        };
    }

    pub(super) fn donate(&mut self, ui: &mut Ui) {
        let donate_text = RichText::new("donate");
        let donate_btn = Label::new(donate_text);
        let donate = ui
            .add(donate_btn.sense(Sense::click()))
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(hints::donate(self.lang));

        if donate.clicked() &&
            let Some(pos) = ui.pointer_interact_pos()
        {
            self.qr_pos.x = pos.x + 10.0;
            self.qr_pos.y = pos.y - 650.0;
            self.show_qr = !self.show_qr;
        }
    }

    pub(super) fn donate_popup(&mut self, ui: &mut Ui) {
        if self.show_qr {
            let resp = Window::new("donate")
                .fixed_size(Vec2::new(320., 160.0))
                .title_bar(false)
                .resizable(false)
                .fixed_pos(self.qr_pos)
                .show(ui, |ui| {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        if let Some(th_btc) = &self.qr_btc &&
                           let Some(th_eth) = &self.qr_eth
                        {
                            ui.vertical(|ui| {
                                let img = Image::new(SizedTexture::new(th_btc.id(), vec2(150.0, 150.0)));
                                let wallet = "bc1qfyt84p8t85pg6597882cr7p04ank2263t7asa6";
                                self.copy_to_clipboard(img, wallet, ui);

                                ui.horizontal(|ui| {
                                    ui.add_space(60.);
                                    ui.colored_label(Color32::ORANGE, "bitcoin");
                                });
                            });

                            ui.vertical(|ui| {
                                let img = Image::new(SizedTexture::new(th_eth.id(), vec2(150.0, 150.0)));
                                let wallet = "0x1991F455084DfF493AC13D0473d92b47A80403F9";
                                self.copy_to_clipboard(img, wallet, ui);

                                ui.horizontal(|ui| {
                                    ui.add_space(50.);
                                    ui.colored_label(Color32::from_rgb(157,167,218), "ethereum");
                                });
                            });
                        }
                    });
                });

            // Hide the popup window if mouse was clicked from outside
            ui.input(|i| {
                if i.pointer.any_click() &&
                    let Some(click_pos) = i.pointer.interact_pos() &&
                    let Some(r) = resp
                {
                    let r_left_top = r.response.rect.left_top() + vec2(-15.0, 20.0);
                    let r_size = r.response.rect.size() + vec2(15.0, 20.0);
                    let popup_rect = Rect::from_min_size(r_left_top, r_size);
                    if !popup_rect.contains(click_pos) {
                        self.show_qr = false;
                    }
                }
            });
        }
    }
    fn copy_to_clipboard(&self, img: Image, text: &str, ui: &mut Ui) {
        if ui
            .add(img.sense(Sense::click()))
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(hints::copy(self.lang))
            .clicked()
        {
            ui.copy_text(text.to_string());
        };
    }

    pub(super) fn save_to(&mut self, ui: &mut Ui) {
        if let Some(tx_id) = &self.save_tx {
            let save_img = Image::new(SizedTexture::new(tx_id.id(), vec2(24., 24.)));
            let save_text = RichText::new(inscriptions::save_to(self.lang).to_lowercase());

            if ui
                .add(save_img.sense(Sense::click()))
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text(save_text)
                .clicked()
                    &&
            let Some(path) = rfd::FileDialog::new()
                .pick_folder()
            {
                self.save_to = path;
                self.save_settings();
            }
        }
    }

    pub(super) fn download(&mut self) {
        // 1. Change name and status of Download button
        if self.dl_started &&
           let Ok(tx) = self.control_tx.try_read()
        {
            let _ = tx.send(Command::Stop);
            return
        }

        self.dl_button_name = inscriptions::cancel(self.lang);
        self.dl_started = true;

        // 2. Create Pause button

        // 3. Main logic
        let form = self.form;
        let genre = self.genre;
        let quantity = self.quantity;
        let period = self.period;
        let lang = self.lang;
        let file_history = self.file_history;
        let lossless = self.lossless;
        let overwrite_files = self.overwrite_files;
        let save_to = self.save_to.clone();
        let client = self.client.clone();
        let db = self.db.clone();
        let common_tx = self.common_tx.clone();
        let control_rx = self.control_rx.clone();

        tokio::spawn(async move {
            let link_params = LinkParams {
                form,
                genre,
                quantity,
                period,
                lang,
                file_history,
                lossless,
                client: client.clone(),
                db,
                common_tx: common_tx.clone(),
            };

            let send = |msg: &str| {
                if let Ok(tx) = common_tx.try_write() {
                    let _ = tx.send(Command::Message(msg.to_owned()));
                }
            };

            match Link::get_all_links(link_params).await {
                Ok(Some(links)) => {
                    let res = download_files(
                            &links,
                            &save_to,
                            client.clone(),
                            overwrite_files,
                            common_tx.clone(),
                            control_rx.clone(),
                          ).await;

                    match res {
                        Ok(Some(Command::Stop)) => {
                            let msg = dictionary::ui_messages::download_canceled(lang);
                            send(msg);
                        }
                        Ok(None) => {
                            let msg = dictionary::ui_messages::all_files_downloaded();
                            send(msg);
                        }
                        Ok(_) => {}
                        Err(e) => {
                            let msg = dictionary::errors::unable_to_download(lang);
                            send(msg);
                            log!("{e}");
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    let msg = dictionary::errors::unable_to_connect(lang);
                    send(msg);
                    log!("{e}");
                }
            }

            if let Ok(tx) = common_tx.try_write() {
                let _ = tx.send(Command::Stop);
            }
        });
    }

    pub(super) fn common_receiver(&mut self, ui: &mut Ui) {
        if let Ok(mut cmd) = self.common_rx.try_write() &&
           let Ok(cmd) = cmd.try_recv()
        {
            match cmd {
                Command::Message(msg) => {
                    self.show_progress = false;
                    self.message = Some(msg);
                    ui.request_repaint();
                }
                Command::Progress(progress, cur, total) => {
                    self.show_progress = true;
                    self.progress = progress;
                    self.current = cur;
                    self.total = total;
                    ui.request_repaint();
                }
                Command::Stop => {
                    self.dl_button_name = inscriptions::download(self.lang);
                    self.dl_started = false;
                }
                _ => {}
            }
        }
    }


    pub(crate) fn progress_bar(&mut self, ui: &mut Ui) {

        // --- 1. Define the area where the UI elements will live ---
        let rect = ui.available_rect_before_wrap();
        let bar_height = 18.0; // Fixed height for the visual bar
        let bar_width = rect.width();

        // Calculate the actual width of the filled portion
        let progress_width = bar_width * self.progress;

        // Draw the filled portion of the bar on top of the background
        let filled_rect = Rect::from_min_size(
            Pos2::new(rect.left(), rect.top()),
            egui::vec2(progress_width, bar_height),
        );
        ui.painter().rect_filled(
            filled_rect,
            3.0,
            Color32::from_rgb(0, 92, 128),
        );

        // --- 2. Draw the Foreground (The Label on top) ---
        let text_color = Color32::LIGHT_GRAY;
        let text = format!("{:.0}% ( {} / {} )", self.progress * 100., self.current, self.total);
        let text_pos = Pos2::new(rect.center().x, rect.center().y);

        ui.painter().text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            text,
            FontId::default(),
            text_color,
        );
        ui.request_repaint();
    }
}
