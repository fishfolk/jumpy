//! Data structures for things like assets and settings that can be serialized and deserialized.

use crate::prelude::*;

use bevy::{reflect::TypeUuid, utils::HashMap};
use bevy_has_load_progress::HasLoadProgress;
use bevy_mod_js_scripting::JsScript;

mod item;
mod localization;
mod map;
mod player;
mod settings;
mod ui;

pub use localization::*;
pub use map::*;
pub use player::*;
pub use settings::*;
pub use ui::*;

#[derive(HasLoadProgress, TypeUuid, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[uuid = "b14f1630-64d0-4bb7-ba3d-e7b83f8a7f62"]
pub struct GameMeta {
    pub players: Vec<String>,
    #[serde(skip)]
    pub player_handles: Vec<Handle<player::PlayerMeta>>,
    pub maps: Vec<String>,
    #[serde(skip)]
    pub map_handles: Vec<Handle<map::MapMeta>>,
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
    pub menu_width: f32,
    // pub music: String,
    // #[serde(skip)]
    // pub music_handle: Handle<AudioSource>,
}

#[derive(HasLoadProgress, Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ImageMeta {
    pub image: String,
    pub image_size: Vec2,
    #[serde(skip)]
    pub image_handle: Handle<Image>,
    /// Egui texture ID may not be valid on all [`ImageMeta`] unless it is added to the egui context
    /// during game load.
    #[serde(skip)]
    pub egui_texture_id: bevy_egui::egui::TextureId,
}
