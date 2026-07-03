use eframe::{App, Frame};
use egui::ComboBox;
use crate::data::consts::{FORMS, GENRES};
use crate::data::dictionary::inscriptions::{download, exit, file_history, last_days, lossless, overwrite_files, period};
use crate::data::dictionary::Lang;

#[derive(Debug)]
pub struct MusicDownloaderApp {
    // Dropdowns
    genre: &'static str,
    form: &'static str,
    last: usize,

    // Toggle values
    file_history: bool,
    overwrite_files: bool,
    period: bool,
    lossless: bool,

    // Options
    language: Lang,
}
impl Default for MusicDownloaderApp {
    fn default() -> Self {
        Self {
            genre: "Techno",
            form: "mixes",
            last: 0,
            file_history: true,
            overwrite_files: false,
            period: true,
            lossless: true,
            language: Lang::En,
        }
    }
}
impl App for MusicDownloaderApp {
    fn ui(&mut self, ctx: &mut egui::Ui, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ComboBox::new("genre", "")
                        .selected_text(&*self.genre)
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            for (text, _) in GENRES.iter() {
                                ui.selectable_value(&mut self.genre, text, *text);
                            }
                        });

                    ComboBox::new("form", "")
                        .selected_text(&*self.form)
                        .show_ui(ui, |ui| {
                            for form in FORMS.iter() {
                                ui.selectable_value(&mut self.form, form, *form);
                            }
                        });

                    ComboBox::new("last", last_days(self.language))
                        .selected_text(self.last.to_string())
                        .show_ui(ui, |ui| {
                            for i in 0..=10 {
                                ui.selectable_value(&mut self.last, i, i.to_string());
                            }
                        });
                });
            });

            // --- Checkbox Row ---
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.toggle_value(&mut self.file_history, file_history(self.language));
                    ui.toggle_value(&mut self.overwrite_files, overwrite_files(self.language));
                    ui.toggle_value(&mut self.period, period(self.language));
                    ui.toggle_value(&mut self.lossless, lossless(self.language));
                })
            });


            // --- Action Buttons Row ---
            ui.horizontal(|ui| {
                // EXIT Button
                if ui.button(exit(self.language)).clicked() {
                    println!("EXIT clicked.");
                }

                // DOWNLOAD Button
                if ui.button(download(self.language)).clicked() {
                    // Displaying the current state to confirm logic works
                    println!("DOWNLOAD initiated with settings: {:?}", self);
                }
            });
        });
    }
}
