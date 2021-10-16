pub mod main_menu;
pub mod pause_menu;
mod style;

use nanoserde::Toml;

pub use style::SkinCollection;

use macroquad::{
    file::load_string,
    texture::{load_texture, Texture2D},
};

use crate::editor::gui::skins::EditorSkinCollection;

#[derive(Debug, Clone)]
pub struct Level {
    pub preview: Texture2D,
    pub map: String,
    pub size: f32,
    pub is_tiled: bool,
    pub is_custom: bool,
}

pub struct GuiResources {
    levels: Vec<Level>,
    pub skins: SkinCollection,
    pub editor_skins: EditorSkinCollection,
}

impl GuiResources {
    pub async fn load(assets_dir: &str) -> GuiResources {
        let mut levels = vec![];

        let levels_str = load_string(&format!("{}/assets/levels/levels.toml", assets_dir))
            .await
            .unwrap();

        let toml = nanoserde::TomlParser::parse(&levels_str).unwrap();

        for level in toml["level"].arr() {
            let mut is_tiled = false;
            if let Some(Toml::Bool(true)) = level.get("is_tiled") {
                is_tiled = true;
            }

            let mut is_custom = false;
            if let Some(Toml::Bool(true)) = level.get("is_custom") {
                is_custom = true;
            }

            levels.push(Level {
                map: format!("{}/{}", assets_dir, level["map"].str()),
                preview: load_texture(&format!("{}/{}", assets_dir, level["preview"].str()))
                    .await
                    .unwrap(),
                size: 0.0,
                is_tiled,
                is_custom,
            })
        }

        GuiResources {
            skins: SkinCollection::new(),
            editor_skins: EditorSkinCollection::new(),
            levels,
        }
    }
}
