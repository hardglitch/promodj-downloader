use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use configparser::ini::Ini;
use eframe::CreationContext;
use eframe::emath::{vec2, Align, Rect};
use eframe::epaint::{Color32, ColorImage, FontFamily};
use eframe::epaint::text::{FontData, FontDefinitions};
use eframe::epaint::textures::TextureOptions;
use egui::{CursorIcon, Image, Label, Layout, RichText, Sense, Ui, Window};
use egui::load::SizedTexture;
use crate::data::consts::{FORMS, GENRES};
use crate::data::dictionary::{hints, inscriptions, Lang};
use crate::log;
use crate::ui::MyApp;
use std::io::Write;

impl MyApp {
    pub fn new(ctx: &CreationContext) -> Self {
        // ctx.egui_ctx.set_pixels_per_point(1.0);

        let mut fonts = FontDefinitions::default();
        fonts.font_data
            .insert(
                "font".to_owned(),
                Arc::new(FontData::from_static(include_bytes!("../../assets/font.ttf")))
            );
        fonts.families.get_mut(&FontFamily::Proportional).unwrap()
            .insert(0, "font".to_owned());

        // egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        ctx.egui_ctx.set_fonts(fonts);

        // let mut style = (*ctx.egui_ctx.global_style()).clone();
        // style.text_styles = [
        //     (TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional)),
        //     (TextStyle::Body, FontId::new(18.0, FontFamily::Proportional)),
        //     (TextStyle::Monospace, FontId::new(18.0, FontFamily::Proportional)),
        //     (TextStyle::Button, FontId::new(20.0, FontFamily::Proportional)),
        //     (TextStyle::Small, FontId::new(16.0, FontFamily::Proportional)),
        // ]
        //     .into();
        // // ctx.egui_ctx.set_global_style(style);
        // ctx.egui_ctx.all_styles_mut(move |style| style.text_styles = style.text_styles.clone());

        let mut app = Self::default();
        app.load_settings();
        app.load_qr_textures(ctx);
        app
    }

    fn save_settings(&mut self) {
        let mut config = Ini::new();
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
            if let Ok(Some(q)) = config.getint("default", "Quantity") {
                self.quantity = q as u64;
            }
        }
    }

    fn load_qr_textures(&mut self, ctx: &CreationContext) {
        let qr_btc = include_bytes!("../../assets/qr_bitcoin.png");
        let image = image::load_from_memory(qr_btc).unwrap().into_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, &image.into_raw());
        let th = ctx.egui_ctx.load_texture("qr_btc", color_image, TextureOptions::default());
        self.qr_btc = Some(th);

        let qr_eth = include_bytes!("../../assets/qr_ethereum.png");
        let image = image::load_from_memory(qr_eth).unwrap().into_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let color_image = ColorImage::from_rgba_unmultiplied(size, &image.into_raw());
        let th = ctx.egui_ctx.load_texture("qr_eth", color_image, TextureOptions::default());
        self.qr_eth = Some(th);
    }

    pub(super) fn copy(&self, text: &str, ui: &mut Ui) {
        let copy_text = RichText::new(egui_phosphor::regular::COPY).size(20.0);
        let copy_btn = Label::new(copy_text);
        if ui
            .add(copy_btn.sense(Sense::click()))
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(hints::copy(self.lang))
            .clicked()
        {
            ui.copy_text(text.to_string());
            // Alternative way
            // ui.output_mut(|o| o.commands.push(OutputCommand::CopyText(text.to_string())));
        };
    }

    pub(super) fn lang_switcher(&mut self, ui: &mut Ui) {
        let lang_text = RichText::new(self.lang.to_string());
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
                .title_bar(false)
                .resizable(false)
                .fixed_pos(self.qr_pos)
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        if let Some(th_btc) = &self.qr_btc &&
                            let Some(th_eth) = &self.qr_eth
                        {
                            ui.add_space(10.0);

                            ui.add(Image::new(SizedTexture::new(th_btc.id(), vec2(200.0, 200.0))));
                            ui.add_space(5.0);
                            ui.colored_label(Color32::ORANGE, "bitcoin");
                            let wallet = "bc1qfyt84p8t85pg6597882cr7p04ank2263t7asa6";
                            ui.label(wallet);
                            self.copy(wallet, ui);

                            ui.add_space(10.0);

                            ui.add(Image::new(SizedTexture::new(th_eth.id(), vec2(200.0, 200.0))));
                            ui.add_space(5.0);
                            ui.colored_label(Color32::from_rgb(157,167,218), "ethereum");
                            let wallet = "0x1991F455084DfF493AC13D0473d92b47A80403F9";
                            ui.label(wallet);
                            self.copy(wallet, ui);
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

    pub(super) fn save_to(&mut self, ui: &mut Ui) {
        let save_text = RichText::new(inscriptions::save_to(self.lang).to_lowercase());
        let save_btn = Label::new(save_text);
        if ui
            .add(save_btn.sense(Sense::click()))
            .on_hover_cursor(CursorIcon::PointingHand)
            // .on_hover_text(hints::export(self.lang))
            .clicked()
            // &&
            // let Some(path) = rfd::FileDialog::new()
            //     .set_file_name("export.txt")
            //     .save_file()
            // &&
            // let Ok(mut file) = File::options()
            //     .create_new(true)
            //     .append(true)
            //     .open(path)
        {
            println!("OK");
        }
    }
}
