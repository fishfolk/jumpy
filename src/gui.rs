pub mod main_menu;
pub mod pause_menu;
mod style;

pub use style::SkinCollection;

use quad_gamepad::ControllerContext;

use crate::editor::gui::skins::EditorSkinCollection;

pub struct GuiResources {
    pub gamepads: ControllerContext,
    pub skins: SkinCollection,
    pub editor_skins: EditorSkinCollection,
}

impl GuiResources {
    pub async fn load(_assets_dir: &str) -> GuiResources {
        GuiResources {
            skins: SkinCollection::new(),
            editor_skins: EditorSkinCollection::new(),
            gamepads: ControllerContext::new().unwrap(),
        }
    }
}
