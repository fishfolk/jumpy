use bevy_egui::egui;

use crate::prelude::*;

pub mod bordered_button;
pub mod bordered_frame;

/// Extensions on [`egui::Ui`] for custom widgets
pub trait EguiUiExt {
    fn themed_label(self, font_meta: &FontMeta, label: &str) -> egui::Response;
}

impl EguiUiExt for &mut egui::Ui {
    fn themed_label(self, font_meta: &FontMeta, label: &str) -> egui::Response {
        self.add(egui::Label::new(
            egui::RichText::new(label)
                .color(font_meta.color.into_egui())
                .font(font_meta.font_id()),
        ))
    }
}

/// Extension trait with helpers for the egui context
pub trait EguiContextExt {
    /// Clear the UI focus
    fn clear_focus(self);
}

impl EguiContextExt for &egui::Context {
    fn clear_focus(self) {
        self.memory().request_focus(egui::Id::null());
    }
}

/// Extension trait with helpers for egui responses
pub trait EguiResponseExt {
    /// Set this response to focused if nothing else is focused
    fn focus_by_default(self, ui: &mut egui::Ui) -> egui::Response;
}

impl EguiResponseExt for egui::Response {
    fn focus_by_default(self, ui: &mut egui::Ui) -> egui::Response {
        if ui.ctx().memory().focus().is_none() {
            ui.ctx().memory().request_focus(self.id);

            self
        } else {
            self
        }
    }
}
