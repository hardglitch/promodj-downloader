use eframe::{App, Frame};
use egui::ComboBox;
use crate::data::consts::GENRES;

#[derive(Debug)]
pub struct MusicDownloaderApp {
    // Input fields state
    genre: &'static str,
    form: &'static str,
    last: usize,

    // Checkbox states
    file_history: bool,
    overwrite_files: bool,
    period: bool,
    lossless: bool,
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
                        .show_ui(ui, |ui| {
                            for (text, _) in &*GENRES {
                                ui.selectable_value(&mut self.genre, *text, *text);
                            }
                        });

                    ComboBox::new("mixes", "")
                        .selected_text(&*self.form)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.form, "mixes", "mixes");
                            ui.selectable_value(&mut self.form, "tracks", "tracks");
                        });

                    ComboBox::new("last_days", "last_days")
                        .selected_text(self.last.to_string())
                        .show_ui(ui, |ui| {
                            for i in 1..10 {
                                ui.selectable_value(&mut self.last, i, i.to_string());
                            }
                        });
                });
            });

            // --- Checkbox Row ---
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.toggle_value(&mut self.file_history, "File History");
                    ui.toggle_value(&mut self.overwrite_files, "Overwrite Files");
                    ui.toggle_value(&mut self.period, "Period");
                    ui.toggle_value(&mut self.lossless, "Lossless");
                })
            });


            // --- Action Buttons Row ---
            ui.horizontal(|ui| {
                // Exit Button
                if ui.button("Exit").clicked() {
                    println!("Exit clicked.");
                }

                // Download Button
                if ui.button("Download").clicked() {
                    // Displaying the current state to confirm logic works
                    println!("Download initiated with settings: {:?}", self);
                }
            });
        });
    }
}
