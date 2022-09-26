use bevy::reflect::TypeUuid;
use bevy_has_load_progress::HasLoadProgress;
use bevy_mod_js_scripting::JsScript;

use crate::prelude::*;

use self::ui::FontMeta;

pub mod localization;
pub mod settings;
pub mod ui;

#[derive(HasLoadProgress, TypeUuid, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[uuid = "b14f1630-64d0-4bb7-ba3d-e7b83f8a7f62"]
pub struct GameMeta {
    pub clear_color: ui::ColorMeta,
    pub camera_height: u32,
    pub translations: localization::TranslationsMeta,
    pub ui_theme: ui::UIThemeMeta,
    pub main_menu: MainMenuMeta,
    pub default_settings: settings::Settings,

    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(skip)]
    pub script_handles: Vec<Handle<JsScript>>,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MainMenuMeta {
    pub title_font: FontMeta,
    pub subtitle_font: FontMeta,
    pub background_image: ImageMeta,
    // pub music: String,
    // #[serde(skip)]
    // pub music_handle: Handle<AudioSource>,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ImageMeta {
    pub image: String,
    pub image_size: Vec2,
    #[serde(skip)]
    pub image_handle: Handle<Image>,
}
