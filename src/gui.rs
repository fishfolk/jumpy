pub mod main_menu;
pub mod pause_menu;
mod style;

pub use style::SkinCollection;

use macroquad::texture::{load_texture, Texture2D};

pub struct GuiResources {
    pub lev01: Texture2D,
    pub lev02: Texture2D,
    pub lev03: Texture2D,
    pub lev04: Texture2D,
    pub lev05: Texture2D,
    pub lev06: Texture2D,

    pub skins: SkinCollection,
}

impl GuiResources {
    pub async fn load() -> GuiResources {
        GuiResources {
            lev01: load_texture("assets/levels/lev01.png").await.unwrap(),
            lev02: load_texture("assets/levels/lev02.png").await.unwrap(),
            lev03: load_texture("assets/levels/lev03.png").await.unwrap(),
            lev04: load_texture("assets/levels/lev04.png").await.unwrap(),
            lev05: load_texture("assets/levels/lev05.png").await.unwrap(),
            lev06: load_texture("assets/levels/lev06.png").await.unwrap(),

            skins: SkinCollection::new(),
        }
    }
}
