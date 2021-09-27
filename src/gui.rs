pub mod main_menu;
pub mod pause_menu;
pub mod editor;
mod style;

use nanoserde::{
    Toml,
    TomlParser,
};

pub use style::SkinCollection;

use quad_gamepad::ControllerContext;

use macroquad::{
    file::load_string,
    texture::{load_texture, Texture2D},
};

#[derive(Debug, Clone)]
struct Level {
    preview: Texture2D,
    map: String,
    size: f32,
    is_custom: bool,
}

pub struct GuiResources {
    levels: Vec<Level>,
    pub gamepads: ControllerContext,
    pub skins: SkinCollection,
}

impl GuiResources {
    pub async fn load() -> GuiResources {
        let mut levels = vec![];
        let levels_str = load_string("assets/levels/levels.toml").await.unwrap();
        let toml = TomlParser::parse(&levels_str).unwrap();

        for level in toml["level"].arr() {
            let mut is_custom = false;
            if let Some(val) = level.get("is_custom") {
                if let Toml::Bool(true) = val {
                    is_custom = true;
                }
            }

            levels.push(Level {
                map: level["map"].str().to_owned(),
                preview: load_texture(level["preview"].str()).await.unwrap(),
                size: 0.,
                is_custom,
            })
        }

        GuiResources {
            skins: SkinCollection::new(),
            gamepads: ControllerContext::new().unwrap(),
            levels,
        }
    }
}
