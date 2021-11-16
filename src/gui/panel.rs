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
    title: Option<String>,
    is_title_centered: bool,
    size: Vec2,
    position: Vec2,
    background_color: Option<Color>,
}

impl Panel {
    const BG_OFFSET: f32 = 12.0;

    pub fn new(id: Id, size: Vec2, position: Vec2) -> Self {
        Panel {
            id,
            title: None,
            is_title_centered: false,
            size,
            position,
            background_color: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_title(self, title: &str, is_centered: bool) -> Self {
        Panel {
            title: Some(title.to_string()),
            is_title_centered: is_centered,
            ..self
        }
    }

    pub fn with_background_color(self, color: Color) -> Self {
        Panel {
            background_color: Some(color),
            ..self
        }
    }

    /// This draws the panel. The callback provided as `f` will be called with the current `Ui` and
    /// the inner size of the panel as arguments. The inner size will be the size of the window,
    /// minus the window margins.
    pub fn ui<F: FnOnce(&mut Ui, Vec2)>(&self, ui: &mut Ui, f: F) {
        {
            let gui_resources = storage::get::<GuiResources>();

            if let Some(background_color) = self.background_color {
                ui.push_skin(&gui_resources.skins.panel_no_bg);

                draw_rectangle(
                    self.position.x + Self::BG_OFFSET,
                    self.position.y + Self::BG_OFFSET,
                    self.size.x - (Self::BG_OFFSET * 2.0),
                    self.size.y - (Self::BG_OFFSET * 2.0),
                    background_color,
                );
            } else {
                ui.push_skin(&gui_resources.skins.panel);
            }
        }

        let _ = widgets::Button::new("")
            .position(self.position)
            .size(self.size)
            .ui(ui);

        let window_margins = vec2(WINDOW_MARGIN_H, WINDOW_MARGIN_V);

        let mut content_position = self.position + window_margins;
        let mut content_size = self.size - (window_margins * 2.0);

        if let Some(title) = &self.title {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.skins.window_header);

            let label_size = ui.calc_size(title);
            let mut label_position = content_position;

            if self.is_title_centered {
                label_position.x += ((self.size.x - label_size.x) / 2.0) - WINDOW_MARGIN_H;
            }

            ui.label(label_position, title);

            content_size.y -= label_size.y;
            content_position.y += label_size.y;

            ui.pop_skin();
        }

        widgets::Group::new(self.id, content_size)
            .position(content_position)
            .ui(ui, |ui| {
                ui.pop_skin();

                f(ui, content_size)
            });
    }
}
