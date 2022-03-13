pub use crate::backend_impl::gui::*;

pub mod combobox;

#[cfg(feature = "macroquad-backend")]
pub mod theme;

#[cfg(feature = "macroquad-backend")]
pub mod checkbox;

#[cfg(feature = "macroquad-backend")]
pub mod menu;

#[cfg(feature = "macroquad-backend")]
pub mod background;

#[cfg(feature = "macroquad-backend")]
pub mod panel;

#[cfg(feature = "macroquad-backend")]
use std::collections::HashMap;

#[cfg(feature = "macroquad-backend")]
use macroquad::prelude::Image;

use serde::{Serialize, Deserialize};

use crate::color::Color;

pub use theme::{
    GuiTheme, BUTTON_FONT_SIZE, BUTTON_MARGIN_H, BUTTON_MARGIN_V, LIST_BOX_ENTRY_HEIGHT,
    SELECTION_HIGHLIGHT_COLOR, WINDOW_BG_COLOR, WINDOW_MARGIN_H, WINDOW_MARGIN_V,
};

pub use combobox::*;
pub use checkbox::*;
pub use panel::*;
pub use menu::*;

pub const NO_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.0);

pub const ELEMENT_MARGIN: f32 = 8.0;