mod credits;
mod game_menu;
mod main_menu;

pub use credits::show_game_credits;
pub use game_menu::{
    close_game_menu, draw_game_menu, is_game_menu_open, open_game_menu, toggle_game_menu,
    GAME_MENU_RESULT_MAIN_MENU, GAME_MENU_RESULT_QUIT,
};
pub use main_menu::MainMenuState;
