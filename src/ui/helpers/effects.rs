use eframe::emath::{pos2, Rect};
use eframe::epaint::{Color32, TextureId};
use egui::{Response, Ui};
use crate::data::consts::FLASH_DURATION_FRAMES;
use crate::ui::helpers::button::ButtonState;
use crate::ui::MyApp;

impl MyApp {
    pub(crate) fn glow_effect(ui: &mut Ui, response: &Response, texture_id: TextureId) {
        let is_hovered = response.hovered();

        let final_color =
            if is_hovered { Color32::from_rgb(210, 210, 210) }
            else { Color32::GRAY };

        ui.painter().image(
            texture_id,
            response.rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            final_color,
        );
    }

    pub(crate) fn enable_flash(&mut self, id: egui::Id) {
        let button_state = ButtonState::new();
        self.buttons
            .entry(id)
            .and_modify(|bs| bs.current_flash_frame = FLASH_DURATION_FRAMES)
            .or_insert(button_state);
    }

    pub(crate) fn flash_effect(&mut self, ui: &mut Ui, response: &Response, texture_id: TextureId) {
        if let Some(bs) = self.buttons.get_mut(&response.id) {
            if bs.current_flash_frame > 0 { bs.current_flash_frame -= 1; } else { return; }

            let opacity = 1. - bs.current_flash_frame as f32 / FLASH_DURATION_FRAMES as f32;
            let top_color = Color32::WHITE;
            let bottom_color =
                if response.hovered() { Color32::from_rgb(210, 210, 210) }
                else { Color32::GRAY };

            // let current_color = bottom_color + (top_color - bottom_color) * opacity; ->
            // -> let current_color = bottom_color * (1 - opacity) + top_color * opacity;
            let k = 1. - opacity;

            #[cfg(target_feature = "sse2")]
            let current_color = || {
                let p1 = crate::utils::simd::simd_multiply_color(&bottom_color, k);
                let p2 = crate::utils::simd::simd_multiply_color(&top_color, opacity);
                crate::utils::simd::simd_add_color(&p1, &p2)
            };

            #[cfg(not(target_feature = "sse2"))]
            let current_color = || {
                let p1 = (
                    (bottom_color.r() as f32 * k) as u8,
                    (bottom_color.g() as f32 * k) as u8,
                    (bottom_color.b() as f32 * k) as u8,
                );
                let p2 = (
                    (top_color.r() as f32 * opacity) as u8,
                    (top_color.g() as f32 * opacity) as u8,
                    (top_color.b() as f32 * opacity) as u8,
                );
                Color32::from_rgb(
                    p1.0 + p2.0,
                    p1.1 + p2.1,
                    p1.2 + p2.2,
                )
            };

            ui.painter().image(
                texture_id,
                response.rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                current_color(),
            );
        }
    }
}