//! This is a builder for panels (windows with static positions)
//! The stock MQ windows do not update their position when screen size changes, even when they
//! are not movable, so we solve this by drawing a non-interactive button, to draw the background,
//! and a group, on top of that, to hold the layout.

use std::ops::Deref;

use crate::color::Color;

use super::{WINDOW_MARGIN_H, WINDOW_MARGIN_V};

use crate::gui::theme::get_gui_theme;
use crate::gui::{widgets, Id, Ui};
use crate::math::{vec2, UVec2, Vec2};
use crate::render::draw_rectangle;
use crate::texture::get_texture;

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
            let gui_theme = get_gui_theme();

            if let Some(background_color) = self.background_color {
                ui.push_skin(&gui_theme.panel_no_bg);

                draw_rectangle(
                    self.position.x + Self::BG_OFFSET,
                    self.position.y + Self::BG_OFFSET,
                    self.size.x - (Self::BG_OFFSET * 2.0),
                    self.size.y - (Self::BG_OFFSET * 2.0),
                    background_color,
                );
            } else {
                ui.push_skin(&gui_theme.panel);
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
            let gui_theme = get_gui_theme();
            ui.push_skin(&gui_theme.window_header);

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

pub struct NewPanel {
    id: Id,
    title: Option<String>,
    is_title_centered: bool,
    size: UVec2,
    position: Vec2,
}

#[allow(dead_code)]
impl NewPanel {
    const TEXTURE_SIZE: f32 = 32.0;

    /// Size is multiplied by 32.0 to get the real pixel size
    pub fn new(id: Id, size: UVec2, position: Vec2) -> Self {
        assert!(size.x >= 2, "Panels horizontal size must be 2 or more");
        assert!(size.y >= 2, "Panels vertical size must be 2 or more");

        NewPanel {
            id,
            title: None,
            is_title_centered: false,
            size,
            position,
        }
    }

    #[allow(dead_code)]
    pub fn with_title(self, title: &str, is_centered: bool) -> Self {
        NewPanel {
            title: Some(title.to_string()),
            is_title_centered: is_centered,
            ..self
        }
    }

    /// This draws the panel. The callback provided as `f` will be called with the current `Ui` and
    /// the inner size of the panel as arguments. The inner size will be the size of the window,
    /// minus the window margins.
    pub fn ui<F: FnOnce(&mut Ui, Vec2)>(&self, ui: &mut Ui, f: F) {
        let full_size = self.size.as_f32() * Self::TEXTURE_SIZE;

        let window_margins = vec2(WINDOW_MARGIN_H, WINDOW_MARGIN_V);

        {
            let upper_lh_corner = get_texture("yellow_board_upper_lh_corner");
            let top = get_texture("yellow_board_top");
            let upper_rh_corner = get_texture("yellow_board_upper_rh_corner");
            let bottom = get_texture("yellow_board_bottom");
            let bg = get_texture("yellow_board_bg");
            let lh_side = get_texture("yellow_board_lh_side");
            let rh_side = get_texture("yellow_board_rh_side");
            let lower_lh_corner = get_texture("yellow_board_lower_lh_corner");
            let lower_rh_corner = get_texture("yellow_board_lower_rh_corner");

            for x in 0..self.size.x {
                for y in 0..self.size.y {
                    let offset = vec2(x as f32 * Self::TEXTURE_SIZE, y as f32 * Self::TEXTURE_SIZE);

                    if x == 0 && y == 0 {
                        widgets::Texture::new(upper_lh_corner.deref().into())
                            .position(self.position + offset)
                            .size(Self::TEXTURE_SIZE, Self::TEXTURE_SIZE)
                            .ui(ui);
                    } else if x == 0 && y == self.size.y - 1 {
                        widgets::Texture::new(lower_lh_corner.deref().into())
                            .position(self.position + offset)
                            .size(Self::TEXTURE_SIZE, Self::TEXTURE_SIZE)
                            .ui(ui);
                    } else if x == self.size.x - 1 && y == 0 {
                        widgets::Texture::new(upper_rh_corner.deref().into())
                            .position(self.position + offset)
                            .size(Self::TEXTURE_SIZE, Self::TEXTURE_SIZE)
                            .ui(ui);
                    } else if x == self.size.x - 1 && y == self.size.y - 1 {
                        widgets::Texture::new(lower_rh_corner.deref().into())
                            .position(self.position + offset)
                            .size(Self::TEXTURE_SIZE, Self::TEXTURE_SIZE)
                            .ui(ui);
                    } else if x == 0 {
                        widgets::Texture::new(lh_side.deref().into())
                            .position(self.position + offset)
                            .size(Self::TEXTURE_SIZE, Self::TEXTURE_SIZE)
                            .ui(ui);
                    } else if x == self.size.x - 1 {
                        widgets::Texture::new(rh_side.deref().into())
                            .position(self.position + offset)
                            .size(Self::TEXTURE_SIZE, Self::TEXTURE_SIZE)
                            .ui(ui);
                    } else if y == 0 {
                        widgets::Texture::new(top.deref().into())
                            .position(self.position + offset)
                            .size(Self::TEXTURE_SIZE, Self::TEXTURE_SIZE)
                            .ui(ui);
                    } else if y == self.size.y - 1 {
                        widgets::Texture::new(bottom.deref().into())
                            .position(self.position + offset)
                            .size(Self::TEXTURE_SIZE, Self::TEXTURE_SIZE)
                            .ui(ui);
                    } else {
                        widgets::Texture::new(bg.deref().into())
                            .position(self.position + offset)
                            .size(Self::TEXTURE_SIZE, Self::TEXTURE_SIZE)
                            .ui(ui);
                    }
                }
            }
        }

        let mut content_position = self.position + window_margins;
        let mut content_size = full_size - (window_margins * 2.0);

        if let Some(title) = &self.title {
            let gui_theme = get_gui_theme();
            ui.push_skin(&gui_theme.window_header);

            let label_size = ui.calc_size(title);
            let mut label_position = content_position;

            if self.is_title_centered {
                label_position.x += ((full_size.x - label_size.x) / 2.0) - WINDOW_MARGIN_H;
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
