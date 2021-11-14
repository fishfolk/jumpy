mod background;
mod create_map;
mod game_menu;
mod main_menu;
mod menu;
mod panel;
mod select_map;
mod style;

pub use style::{
    SkinCollection, BUTTON_FONT_SIZE, BUTTON_MARGIN_H, BUTTON_MARGIN_V, WINDOW_MARGIN_H,
    WINDOW_MARGIN_V,
};

use crate::editor::gui::skins::EditorSkinCollection;

pub use background::{draw_main_menu_background, Background};
pub use create_map::show_create_map_menu;
pub use game_menu::{
    close_game_menu, draw_game_menu, is_game_menu_open, open_game_menu, toggle_game_menu,
    GAME_MENU_RESULT_MAIN_MENU, GAME_MENU_RESULT_QUIT,
};
pub use main_menu::{show_main_menu, MainMenuResult};
pub use menu::{Menu, MenuEntry, MenuResult};
pub use panel::Panel;
pub use select_map::show_select_map_menu;

pub struct GuiResources {
    pub skins: SkinCollection,
    pub editor_skins: EditorSkinCollection,
}

impl GuiResources {
    pub async fn load(_assets_dir: &str) -> GuiResources {
        GuiResources {
            skins: SkinCollection::new(),
            editor_skins: EditorSkinCollection::new(),
        }
    }
}
