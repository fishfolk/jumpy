//! This is a builder for panels (windows with static positions)
//! The stock MQ windows do not update their position when screen size changes, even when they
//! are not movable, so we solve this by drawing a non-interactive button, to draw the background,
//! and a group, on top of that, to hold the layout.

use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{widgets, Id, Ui},
};

use super::GuiResources;

// This must be updated if the window margins are changed in the gui styles
const WINDOW_MARGIN: f32 = 22.0;

pub struct Panel {
    id: Id,
    size: Vec2,
    position: Vec2,
}

impl Panel {
    pub fn new(id: Id, size: Vec2, position: Vec2) -> Self {
        Panel { id, size, position }
    }

    pub fn ui<F: FnOnce(&mut Ui)>(&self, ui: &mut Ui, f: F) {
        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.skins.panel_group);
        }

        let _ = widgets::Button::new("")
            .position(self.position)
            .size(self.size)
            .ui(ui);

        let position = self.position + vec2(WINDOW_MARGIN, WINDOW_MARGIN);
        let size = self.size - vec2(WINDOW_MARGIN * 2.0, WINDOW_MARGIN * 2.0);

        widgets::Group::new(self.id, size)
            .position(position)
            .ui(ui, |ui| {
                ui.pop_skin();

                f(ui)
            });
    }
}
