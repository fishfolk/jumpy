use macroquad::{
    ui::{
        Id,
        Ui,
        widgets::{
            self,
            Window,
        },
    },
    experimental::{
        collections::storage,
    },
    prelude::*,
};

use crate::gui::GuiResources;

pub enum WindowPosition {
    Centered,
    Custom(Vec2),
}

// This should be moved out of editor and into the root ui module, if we
// decide to formalize ui style by using builders, like this...
pub struct WindowBuilder {
    title: Option<String>,
    size: Vec2,
    position: WindowPosition,
    is_static: bool,
}

impl WindowBuilder {
    pub fn new(size: Vec2) -> Self {
        WindowBuilder {
            title: None,
            size,
            position: WindowPosition::Centered,
            is_static: true,
        }
    }

    pub fn with_title(self, title: &str) -> Self {
        WindowBuilder {
            title: Some(title.to_string()),
            ..self
        }
    }

    pub fn with_position(self, position: WindowPosition, is_static: bool) -> Self {
        WindowBuilder {
            position,
            is_static,
            ..self
        }
    }

    pub fn build<F: FnOnce(&mut Ui)>(&self, id: Id, ui: &mut Ui, f: F) {
        let position =
            match self.position {
                WindowPosition::Centered => {
                    let x = (screen_width() - self.size.x) / 2.0;
                    let y = (screen_height() - self.size.y) / 2.0;
                    vec2(x, y)
                }
                WindowPosition::Custom(position) => {
                    position
                }
            };

        Window::new(id, position, self.size)
            .titlebar(false)
            .movable(self.is_static == false)
            .ui(ui, |ui| {
                if let Some(title) = &self.title {
                    let gui_resources = storage::get::<GuiResources>();
                    ui.push_skin(&gui_resources.skins.editor_window_header_skin);
                    ui.label(None, title);
                    ui.pop_skin();
                }

                f(ui);
            });
    }
}