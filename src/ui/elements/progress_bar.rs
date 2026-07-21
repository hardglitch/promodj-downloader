use eframe::emath::{Pos2, Rect};
use eframe::epaint::{Color32, FontId};
use egui::Ui;
use crate::ui::MyApp;

impl MyApp {
    pub(crate) fn progress_bar(&mut self, ui: &mut Ui) {

        // --- 1. Define the area where the UI elements will live ---
        let rect = ui.available_rect_before_wrap();
        let bar_height = 20.0; // Fixed height for the visual bar
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