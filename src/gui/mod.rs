mod create_map;
mod game_menu;
mod main_menu;
mod select_map;
mod style;

pub use style::SkinCollection;

use crate::editor::gui::skins::EditorSkinCollection;

pub use create_map::show_create_map_menu;
pub use game_menu::{show_game_menu, GameMenuResult};
pub use main_menu::{show_main_menu, MainMenuResult};
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
