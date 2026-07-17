use crate::data::consts::{FORMS, GENRES};
use crate::data::dictionary;
use crate::data::dictionary::{hints, inscriptions, Lang};
use crate::db::dbcore::Database;
use crate::log;
use crate::logic::file::download_files;
use crate::logic::proxy::ProxyType;
use crate::logic::search::{Link, LinkParams};
use crate::logic::Command;
use crate::ui::MyApp;
use configparser::ini::Ini;
use eframe::emath::{vec2, Align, Rect};
use eframe::epaint::text::{FontData, FontDefinitions};
use eframe::epaint::textures::TextureOptions;
use eframe::epaint::{Color32, ColorImage, FontFamily};
use eframe::CreationContext;
use egui::load::SizedTexture;
use egui::{pos2, ComboBox, CursorIcon, FontId, Image, Label, Layout, Pos2, Response, RichText, Sense, TextStyle, TextureId, Ui, Vec2, Window};
use image::{GenericImageView, ImageBuffer};
use image::{ImageError, ImageResult, Rgba};
use std::io::Write;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use strum::IntoEnumIterator;

enum Color {
    LightGray,
    White,
}
impl MyApp {
    pub fn new(ctx: &CreationContext, db: Arc<Database>) -> Self {

        // Default settings of the App
        let mut app = Self::default();

        // Set db
        app.db = Some(db);

        // Apply a custom font
        let mut fonts = FontDefinitions::default();
        fonts.font_data
            .insert("font".to_owned(), Arc::new(FontData::from_static(include_bytes!("../../assets/font.ttf"))));
        fonts.families.get_mut(&FontFamily::Proportional)
            .into_iter().for_each(|font| { font.insert(0, "font".to_owned()); });
        ctx.egui_ctx.set_fonts(fonts);

        // Last days for Header
        app.last_download_days = Self::last_download_days().unwrap_or_default();

        // Try load the proxy settings
        if let Err(e) = app.load_proxy() { log!("Failed to load the proxy settings: {e}") }

        // Try load textures
        app.load_settings();
        if let Err(e) = app.load_textures(ctx) { log!("Failed to load textures: {e}") }
        app
    }

    pub(super) fn save_settings(&self) {
        let mut config = Ini::new();

        config.set("default", "LastDownload", Some(self.last_download.to_string()));
        config.set("default", "Language", Some(self.lang.encode().to_owned()));

        let s = self.save_to.clone().into_string().ok();
        config.set("default", "DownloadDirectory", s);

        config.set("default", "Genre", Some(self.genre.to_owned()));
        config.set("default", "Form", Some(self.form.to_owned()));
        config.set("default", "Lossless", Some(self.lossless.to_string()));
        config.set("default", "Period", Some(self.period.to_string()));
        config.set("default", "OverwriteFiles", Some(self.overwrite_files.to_string()));
        config.set("default", "FileHistory", Some(self.file_history.to_string()));

        if let Ok(quantity) = self.quantity.parse::<u16>() {
            config.set("default", "Quantity", Some(quantity.to_string()));
        }
        config.set("default", "Proxy", Some(self.use_proxy.to_string()));

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
                self.quantity = (q as usize).to_string();
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
        let color_image = Self::process_image(img, Color::White)?;
        let th = ctx.egui_ctx.load_texture("save_tx", color_image, TextureOptions::default());
        self.save_tx = Some(th);

        let img = include_bytes!("../../assets/pause.ico");
        let color_image = Self::process_image(img, Color::LightGray)?;
        let th = ctx.egui_ctx.load_texture("pause_tx", color_image, TextureOptions::default());
        self.pause_tx = Some(th);

        let img = include_bytes!("../../assets/play.ico");
        let color_image = Self::process_image(img, Color::LightGray)?;
        let th = ctx.egui_ctx.load_texture("play_tx", color_image, TextureOptions::default());
        self.play_tx = Some(th);

        let img = include_bytes!("../../assets/gear.png");
        let color_image = Self::process_image(img, Color::White)?;
        let th = ctx.egui_ctx.load_texture("settings_tx", color_image, TextureOptions::default());
        self.proxy_settings_tx = Some(th);

        Ok(())
    }

    fn glow_effect(ui: &mut Ui, response: &Response, texture_id: TextureId) {
        let is_hovered = response.hovered();
        let final_color = if is_hovered { Color32::WHITE } else { Color32::GRAY };
        ui.painter().image(
            texture_id,
            response.rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            final_color,
        );
    }

    fn process_image(img: &[u8], color: Color) -> Result<ColorImage, ImageError> {
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
                    let new_pixel = match color {
                        Color::LightGray => Rgba([200, 200, 200, alpha]),
                        Color::White => Rgba([255, 255, 255, alpha]),
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

    pub(super) fn last_download_days() -> Option<u64> {
        let mut config = Ini::new();
        if config.load("settings.ini").is_ok() &&
            let Ok(Some(last_ts)) = config.getuint("default", "LastDownload") &&
            let Ok(ts) = std::time::SystemTime::now().duration_since(UNIX_EPOCH)
        {
            let days = ts.as_secs().saturating_sub(last_ts).saturating_div(3600 * 24);
            return Some(days)
        }
        None
    }
    pub(super) fn window_title(&mut self) -> String {
        let template = inscriptions::window_title(self.lang);
        template.replace('_', &self.last_download_days.to_string())
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

        self.qr_rect = Some(donate.rect);

        if donate.clicked() &&
           let Some(pos) = ui.pointer_interact_pos()
        {
            self.qr_pos.x = pos.x + 10.0;
            self.qr_pos.y = pos.y - 190.0;
            self.show_qr = !self.show_qr;
        }
    }
    pub(super) fn donate_popup(&mut self, ui: &mut Ui) {
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
                // dbg!(&r.response.rect, &self.qr_rect, &click_pos);
                if !r.response.rect.contains(click_pos) &&
                    self.qr_rect.is_some_and(|r| !r.contains(click_pos))
                {
                    self.show_qr = false;
                }
            }
        });
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

    pub(super) fn proxy_settings(&mut self, ui: &mut Ui) {
        if let Some(tx_id) = &self.proxy_settings_tx {
            let img = Image::new(SizedTexture::new(tx_id.id(), vec2(16., 16.)));
            let text = RichText::new(hints::proxy_settings(self.lang));

            let button = ui.add(img.sense(Sense::click()));
            Self::glow_effect(ui, &button, tx_id.id());
            self.proxy_rect = Some(button.rect);

            if button
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text(text)
                .clicked()
                    &&
                let Some(pos) = ui.pointer_interact_pos()
            {
                self.proxy_window_pos.x = pos.x - 390.0;
                self.proxy_window_pos.y = pos.y + 10.0;
                self.show_proxy_window = !self.show_proxy_window;
            }
        }
    }
    pub(super) fn proxy_popup(&mut self, ui: &mut Ui) {
        let resp = Window::new("proxy")
            .fixed_size(Vec2::new(410., 100.0))
            .title_bar(false)
            .resizable(false)
            .fixed_pos(self.proxy_window_pos)
            .show(ui, |ui| {
                ui.add_space(5.);

                ui.horizontal(|ui| {

                    // 1. Proxy type | Host | Port
                    // 1-1. Proxy type
                    ComboBox::new("proxy_type", "")
                        .selected_text(self.proxy_type.to_string().to_lowercase())
                        .show_ui(ui, |ui| {
                            for proxy_type in ProxyType::iter() {
                                ui.selectable_value(&mut self.proxy_type, proxy_type, proxy_type.to_string());
                            }
                        });

                    // 1-2. Host
                    let state_before = self.proxy_host.clone();
                    let widget = egui::widgets::TextEdit::singleline(&mut self.proxy_host)
                        .desired_width(200.);
                    ui.add(widget);
                    if self.proxy_host == state_before ||
                       self.proxy_host.parse::<IpAddr>().is_err()
                    {
                        self.proxy_host = state_before
                    }

                    ui.label(":");

                    // 1-3. Port
                    let state_before = self.proxy_port.clone();
                    let widget = egui::widgets::TextEdit::singleline(&mut self.proxy_port)
                        .desired_width(50.);
                    ui.add(widget);
                    if self.proxy_port == state_before ||
                       self.proxy_port.parse::<u16>().is_err()
                    {
                        self.proxy_port = state_before
                    }

                });

                ui.add_space(5.);

                // 2. Auth
                // 2-1. Login (disabled if not http_auth)
                ui.horizontal(|ui| {
                    if let Ok(mut buf) = self.proxy_login.try_write() {
                       let enabled = matches!(self.proxy_type, ProxyType::HttpAuth);
                       let widget = egui::widgets::TextEdit::singleline(&mut *buf)
                            .desired_width(ui.available_width())
                            .hint_text(hints::login(self.lang))
                            .font(TextStyle::Heading)
                            .desired_rows(1)
                            .interactive(enabled);

                        ui.add(widget);
                    }
                });

                // 2-2. Password (disabled if not http_auth)
                ui.horizontal(|ui| {
                    if let Ok(mut buf) = self.proxy_password.try_write() {
                        let enabled = matches!(self.proxy_type, ProxyType::HttpAuth);
                        let widget = egui::widgets::TextEdit::singleline(&mut *buf)
                            .desired_width(ui.available_width())
                            .hint_text(hints::password(self.lang))
                            .font(TextStyle::Heading)
                            .desired_rows(1)
                            .password(true)
                            .interactive(enabled);

                        ui.add(widget);
                    }
                });
            });

        // Hide the popup window if mouse was clicked from outside
        ui.input(|i| {
            if i.pointer.any_click() &&
               let Some(click_pos) = i.pointer.interact_pos() &&
               let Some(r) = resp
            {
                // dbg!(&r.response.rect, &self.proxy_rect, &click_pos);
                if !r.response.rect.contains(click_pos) &&
                   self.proxy_rect.is_some_and(|r| !r.contains(click_pos))
                {
                    self.show_proxy_window = false;
                    self.save_proxy();
                }
            }
        });
    }

    pub(super) fn save_to(&mut self, ui: &mut Ui) {
        if let Some(tx_id) = &self.save_tx {
            let img = Image::new(SizedTexture::new(tx_id.id(), vec2(24., 24.)));
            let text = RichText::new(inscriptions::save_to(self.lang));

            let button = ui.add(img.sense(Sense::click()));
            Self::glow_effect(ui, &button, tx_id.id());

            if button
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text(text)
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

    pub(super) fn progress_bar(&mut self, ui: &mut Ui) {

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

    pub(super) fn download(&mut self) {

        // 1. Change name and status of the 'Download' button
        if self.dl_started &&
           let Ok(tx) = self.control_tx.try_read()
        {
            let _ = tx.send(Command::Stop);
            return
        }

        self.dl_button_name = inscriptions::cancel;
        self.dl_started = true;

        // 2. Main logic
        let form = self.form;
        let genre = self.genre;
        let quantity = if let Ok(q) = self.quantity.parse::<u16>() { q as usize } else { return };
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
                db: db.clone(),
                common_tx: common_tx.clone(),
            };

            let send = |msg: &str| {
                if let Ok(tx) = common_tx.try_read() {
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
                            db,
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
                Ok(None) => {
                    let msg = dictionary::ui_messages::matching_files_not_found(lang);
                    send(msg);
                }
                Err(e) => {
                    let msg = dictionary::errors::unable_to_connect(lang);
                    send(msg);
                    log!("{e}");
                }
            }

            if let Ok(tx) = common_tx.try_read() {
                let _ = tx.send(Command::Stop);
            }
        });
    }
    pub(super) fn pause(&mut self, ui: &mut Ui) {
        let tx_handle = if self.dl_paused { &self.play_tx } else { &self.pause_tx };
        if let Some(tx_id) = tx_handle {
            let img = Image::new(SizedTexture::new(tx_id.id(), vec2(30., 30.)));
            let hint =
                if self.dl_paused { hints::resume(self.lang) }
                else { hints::pause(self.lang) };

            let text = RichText::new(hint);

            if ui
                .add(img.sense(Sense::click()))
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text(text)
                .clicked()
                    &&
                let Ok(tx) = self.control_tx.try_read()
            {
                if self.dl_paused {
                    let _ = tx.send(Command::Start);
                    self.dl_paused = false;
                }
                else {
                    let _ = tx.send(Command::Pause);
                    self.dl_paused = true;
                }
                ui.request_repaint();
            }
        }
    }

    pub(super) fn common_receiver(&mut self) {
        if let Ok(mut cmd) = self.common_rx.try_write() &&
           let Ok(cmd) = cmd.try_recv()
        {
            match cmd {
                Command::Message(msg) => {
                    self.show_progress = false;
                    self.message = Some(msg);
                }
                Command::Progress(progress, cur, total) => {
                    self.show_progress = true;
                    self.progress = progress;
                    self.current = cur;
                    self.total = total;
                }
                Command::Stop => {
                    self.dl_button_name = inscriptions::download;
                    self.dl_started = false;
                }
                Command::Success => {
                    self.last_download_days = 0;
                    self.save_settings();
                }
                _ => {}
            }
        }
    }
}
