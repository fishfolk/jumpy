use macroquad::{
    ui::Ui,
    prelude::*,
};

use super::{
    WindowBuilder,
    WindowPosition,
};

pub struct ConfirmDialog {
    size: Vec2,
    body: Vec<String>,
}

impl ConfirmDialog {
    const WINDOW_TITLE: &'static str = "Please Confirm";
    const CONFIRM_LABEL: &'static str = "Ok";
    const CANCEL_LABEL: &'static str = "Cancel";

    pub fn new(size: Vec2, body: &[&str]) -> Self {
        let body = body
            .into_iter()
            .map(|line| line.to_string())
            .collect();

        ConfirmDialog {
            size,
            body,
        }
    }

    pub fn get_rect(&self) -> Rect {
        let position = WindowPosition::Centered.to_absolute(self.size);
        Rect::new(position.x, position.y, self.size.x, self.size.y)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        let rect = self.get_rect();
        rect.contains(point)
    }

    pub fn draw(&self, ui: &mut Ui) -> Option<bool> {
        let mut res = None;

        WindowBuilder::new(self.size)
            .with_title(Self::WINDOW_TITLE)
            .with_position(WindowPosition::Centered, true)
            .build(ui, |ui| {
                for line in &self.body {
                    ui.label(None, line);
                }

                ui.separator();
                ui.separator();
                ui.separator();
                ui.separator();

                if ui.button(None, Self::CONFIRM_LABEL) {
                    res = Some(true);
                }

                if ui.button(None, Self::CANCEL_LABEL) {
                    res = Some(false);
                }
            });

        ui.pop_skin();

        res
    }
}

