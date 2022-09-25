#![allow(unused)] // TODO: Remove when used

use bevy_egui::egui;

use crate::metadata::ui::ProgressBarMeta;

use super::bordered_frame::BorderedFrame;

pub struct ProgressBar<'a> {
    pub theme: &'a ProgressBarMeta,
    pub progress: f32,
    pub min_width: f32,
}

impl<'a> ProgressBar<'a> {
    #[must_use = "You must call .show() to render the progress bar"]
    pub fn new(theme: &'a ProgressBarMeta, progress: f32) -> Self {
        Self {
            theme,
            progress,
            min_width: 0.0,
        }
    }

    #[must_use = "You must call .show() to render the progress bar"]
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let bg = &self.theme.background_image;
        let size = bg.image_size.as_vec2() * bg.scale;
        let size = egui::vec2(size.x, size.y);
        let size = size.max(egui::vec2(self.min_width, self.theme.height));

        let (rect, response) = ui.allocate_at_least(size, egui::Sense::hover());

        let frame = BorderedFrame::new(&self.theme.background_image).paint(rect);
        ui.painter().add(frame);

        let b = bg.border_size;
        let inner_rect_min = rect.min + egui::vec2(bg.scale * b.left, bg.scale * b.top);
        let inner_rect_size = egui::vec2(
            self.progress.max(0.0) * (size.x - (b.left + b.right) * bg.scale),
            size.y - (b.top + b.bottom) * bg.scale,
        );
        let inner_rect = egui::Rect::from_min_size(inner_rect_min, inner_rect_size);
        let bar = BorderedFrame::new(&self.theme.progress_image).paint(inner_rect);
        ui.painter().add(bar);

        response
    }
}
