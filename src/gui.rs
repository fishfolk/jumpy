pub mod main_menu;
pub mod pause_menu;
mod style;

pub use style::SkinCollection;

use quad_gamepad::ControllerContext;

use macroquad::{
    file::load_string,
    texture::{load_texture, Texture2D},
};

struct Level {
    preview: Texture2D,
    map: String,
    size: f32,
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
        let toml = nanoserde::TomlParser::parse(&levels_str).unwrap();

        for level in toml["level"].arr() {
            levels.push(Level {
                map: level["map"].str().to_owned(),
                preview: load_texture(level["preview"].str()).await.unwrap(),
                size: 0.,
            })
        }

        GuiResources {
            skins: SkinCollection::new(),
            gamepads: ControllerContext::new().unwrap(),
            levels,
        }
    }
}
