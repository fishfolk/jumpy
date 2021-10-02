use macroquad::{
    ui::{
        Id,
        Ui,
        widgets::{
            self,
            Window,
        },
        hash,
    },
    experimental::{
        collections::storage,
    },
    prelude::*,
};

use crate::gui::GuiResources;

use crate::editor::{
    EditorAction,
    EditorDrawParams,
};

#[derive(Debug, Copy, Clone)]
pub enum WindowPosition {
    Centered,
    Custom(Vec2),
}

impl WindowPosition {
    pub fn to_absolute(&self, size: Vec2) -> Vec2 {
        match self {
            Self::Centered => {
                let screen_size = vec2(screen_width(), screen_height());
                (screen_size - size) / 2.0
            }
            Self::Custom(position) => {
                *position
            }
        }
    }
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

    pub fn build<F: FnOnce(&mut Ui)>(&self, ui: &mut Ui, f: F) {
        let position = self.position.to_absolute(self.size);

        Window::new(hash!(), position, self.size)
            .titlebar(false)
            .movable(self.is_static == false)
            .ui(ui, |ui| {
                if let Some(title) = &self.title {
                    let gui_resources = storage::get::<GuiResources>();
                    ui.push_skin(&gui_resources.editor_skins.window_header);
                    ui.label(None, title);
                    ui.pop_skin();
                }

                f(ui);
            });
    }
}