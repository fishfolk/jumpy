//! This is a builder for panels (windows with static positions)
//! The stock MQ windows do not update their position when screen size changes, even when they
//! are not movable, so we solve this by drawing a non-interactive button, to draw the background,
//! and a group, on top of that, to hold the layout.

use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{widgets, Id, Ui},
};

use super::{GuiResources, WINDOW_MARGIN_H, WINDOW_MARGIN_V};

pub struct Panel {
    id: Id,
    size: Vec2,
    position: Vec2,
}

impl Panel {
    pub fn new(id: Id, size: Vec2, position: Vec2) -> Self {
        Panel { id, size, position }
    }

    /// This draws the panel. The callback provided as `f` will be called with the current `Ui` and
    /// the inner size of the panel as arguments. The inner size will be the size of the window,
    /// minus the window margins.
    pub fn ui<F: FnOnce(&mut Ui, Vec2)>(&self, ui: &mut Ui, f: F) {
        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.skins.panel_group);
        }

        let _ = widgets::Button::new("")
            .position(self.position)
            .size(self.size)
            .ui(ui);

        let position = self.position + vec2(WINDOW_MARGIN_H, WINDOW_MARGIN_V);
        let size = self.size - vec2(WINDOW_MARGIN_H * 2.0, WINDOW_MARGIN_V * 2.0);

        widgets::Group::new(self.id, size)
            .position(position)
            .ui(ui, |ui| {
                ui.pop_skin();

                f(ui, size)
            });
    }
}
