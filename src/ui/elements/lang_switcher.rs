use egui::{CursorIcon, Label, RichText, Sense, Ui};
use crate::data::dictionary::{hints, Lang};
use crate::ui::MyApp;

impl MyApp {
    pub(crate) fn lang_switcher(&mut self, ui: &mut Ui) {
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
}