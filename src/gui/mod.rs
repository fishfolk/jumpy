mod background;
mod checkbox;
mod create_map;
mod credits;
mod game_menu;
mod main_menu;
mod menu;
mod panel;
mod select_character;
mod select_map;
mod style;

use macroquad::prelude::*;

pub use style::{
    SkinCollection, BUTTON_FONT_SIZE, BUTTON_MARGIN_H, BUTTON_MARGIN_V, LIST_BOX_ENTRY_HEIGHT,
    SELECTION_HIGHLIGHT_COLOR, WINDOW_BG_COLOR, WINDOW_MARGIN_H, WINDOW_MARGIN_V,
};

pub use background::{draw_main_menu_background, Background};
pub use checkbox::Checkbox;
pub use create_map::show_create_map_menu;
pub use credits::show_game_credits;
pub use game_menu::{
    close_game_menu, draw_game_menu, is_game_menu_open, open_game_menu, toggle_game_menu,
    GAME_MENU_RESULT_MAIN_MENU, GAME_MENU_RESULT_QUIT,
};
pub use main_menu::{show_main_menu, MainMenuResult};
pub use menu::{Menu, MenuEntry, MenuResult};
pub use panel::{NewPanel, Panel};
pub use select_character::show_select_characters_menu;
pub use select_map::show_select_map_menu;

pub const NO_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.0);

pub const ELEMENT_MARGIN: f32 = 8.0;

pub struct GuiResources {
    pub skins: SkinCollection,
}

impl GuiResources {
    pub fn new() -> GuiResources {
        GuiResources {
            skins: SkinCollection::new(),
        }
    }
}
