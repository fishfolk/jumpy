pub use crate::backend_impl::gui::*;

#[cfg(feature = "macroquad-backend")]
pub mod combobox;

#[cfg(feature = "macroquad-backend")]
pub mod theme;

#[cfg(feature = "macroquad-backend")]
pub mod checkbox;

#[cfg(feature = "macroquad-backend")]
pub mod menu;

pub mod background;

#[cfg(feature = "macroquad-backend")]
pub mod panel;

#[cfg(feature = "macroquad-backend")]
use std::collections::HashMap;

#[cfg(feature = "macroquad-backend")]
use macroquad::prelude::Image;

use serde::{Deserialize, Serialize};

use crate::color::Color;

#[cfg(feature = "macroquad-backend")]
pub use theme::{
    get_gui_theme, rebuild_gui_theme, GuiTheme, BUTTON_FONT_SIZE, BUTTON_MARGIN_H, BUTTON_MARGIN_V,
    LIST_BOX_ENTRY_HEIGHT, SELECTION_HIGHLIGHT_COLOR, WINDOW_BG_COLOR, WINDOW_MARGIN_H,
    WINDOW_MARGIN_V,
};

#[cfg(feature = "macroquad-backend")]
pub use checkbox::*;
#[cfg(feature = "macroquad-backend")]
pub use combobox::*;
#[cfg(feature = "macroquad-backend")]
pub use menu::*;
#[cfg(feature = "macroquad-backend")]
pub use panel::*;

pub const ELEMENT_MARGIN: f32 = 8.0;
