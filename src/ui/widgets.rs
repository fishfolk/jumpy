use bevy_egui::egui;

use crate::metadata::ui::FontMeta;

pub mod bordered_button;
pub mod bordered_frame;
pub mod progress_bar;

pub trait EguiUIExt {
    fn themed_label(self, font_meta: &FontMeta, label: &str) -> egui::Response;
}

impl EguiUIExt for &mut egui::Ui {
    fn themed_label(self, font_meta: &FontMeta, label: &str) -> egui::Response {
        self.add(egui::Label::new(
            egui::RichText::new(label)
                .color(font_meta.color)
                .font(font_meta.font_id()),
        ))
    }
}
