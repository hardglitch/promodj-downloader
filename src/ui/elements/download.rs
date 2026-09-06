use eframe::emath::vec2;
use egui::{CursorIcon, Image, RichText, Sense, Ui};
use egui::load::SizedTexture;
use log::log;
use crate::data::dictionary;
use crate::data::dictionary::{hints, inscriptions};
use crate::logic::Command;
use crate::logic::file::download_files;
use crate::logic::search::{Link, LinkParams};
use crate::ui::MyApp;

impl MyApp {
    pub(crate) fn download(&mut self) {

        // 1. Change name and status of the 'Download' button
        if self.dl_started &&
            let Ok(tx) = self.control_tx.try_read()
        {
            let _ = tx.send(Command::Stop);
            return
        }

        self.dl_button_name = inscriptions::cancel;

        // 2. Main logic
        let form = self.form;
        let genre = self.genre;
        let quantity = if let Ok(q) = self.quantity.parse::<u16>() { q as usize } else { return };
        let period = self.period;
        let lang = self.lang;
        let file_history = self.file_history;
        let lossless = self.lossless;
        let use_xf_words = self.use_exclusion_filter;
        let xf_words = self.xf_words.clone();
        let use_if_words = self.use_inclusion_filter;
        let if_words = self.if_words.clone();
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
                use_xf_words,
                xf_words,
                use_if_words,
                if_words,
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
                    if let Ok(tx) = common_tx.try_read() {
                        let _ = tx.send(Command::Started);
                    }
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
                Ok(None) => {}
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

    pub(crate) fn pause(&mut self, ui: &mut Ui) {
        let tx_handle = if self.dl_paused { &self.play_tx } else { &self.pause_tx };
        if let Some(tx_id) = tx_handle {
            let img = Image::new(SizedTexture::new(tx_id.id(), vec2(30., 30.)));
            let hint =
                if self.dl_paused { hints::resume(self.lang) }
                else { hints::pause(self.lang) };

            let text = RichText::new(hint);
            let button = ui.add(img.sense(Sense::click()));
            Self::glow_effect(ui, &button, tx_id.id());

            if button
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
}