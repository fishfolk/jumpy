mod create_map;
mod credits;
mod game_menu;
mod main_menu;
mod select_character;
mod select_map;

use ff_core::prelude::*;
pub use create_map::show_create_map_menu;
pub use credits::show_game_credits;
pub use game_menu::{
    close_game_menu, draw_game_menu, is_game_menu_open, open_game_menu, toggle_game_menu,
    GAME_MENU_RESULT_MAIN_MENU, GAME_MENU_RESULT_QUIT,
};
pub use main_menu::{show_main_menu, MainMenuResult};
pub use select_character::show_select_characters_menu;
pub use select_map::show_select_map_menu;
