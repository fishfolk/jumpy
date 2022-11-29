//! Data structures for things like assets and settings that can be serialized and deserialized.

use crate::prelude::*;

use bevy::{reflect::TypeUuid, utils::HashMap};
use bevy_has_load_progress::HasLoadProgress;
use bevy_kira_audio::AudioSource;

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

pub struct MetadataPlugin;

impl Plugin for MetadataPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MapMetadataPlugin)
            .add_plugin(PlayerMetadataPlugin);
    }
}

#[derive(HasLoadProgress, TypeUuid, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[uuid = "b14f1630-64d0-4bb7-ba3d-e7b83f8a7f62"]
pub struct GameMeta {
    pub players: Vec<String>,
    #[serde(skip)]
    pub player_handles: Vec<AssetHandle<player::PlayerMeta>>,
    pub stable_maps: Vec<String>,
    #[serde(skip)]
    pub stable_map_handles: Vec<AssetHandle<map::MapMeta>>,
    pub experimental_maps: Vec<String>,
    #[serde(skip)]
    pub experimental_map_handles: Vec<AssetHandle<map::MapMeta>>,
    pub clear_color: ui::ColorMeta,
    pub camera_height: u32,
    pub translations: localization::TranslationsMeta,
    pub ui_theme: ui::UIThemeMeta,
    pub main_menu: MainMenuMeta,
    pub default_settings: settings::Settings,
    pub physics: PhysicsMeta,
    pub playlist: Vec<String>,
    #[serde(skip)]
    pub playlist_handles: Vec<Handle<AudioSource>>,
    // /// Scripts that run on both the server and the client
    // #[serde(default)]
    // pub scripts: Vec<String>,
    // #[serde(skip)]
    // pub script_handles: Vec<AssetHandle<JsScript>>,
}

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PhysicsMeta {
    pub terminal_velocity: f32,
    pub friction_lerp: f32,
    pub stop_threshold: f32,
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
    pub image_handle: AssetHandle<Image>,
    /// Egui texture ID may not be valid on all [`ImageMeta`] unless it is added to the egui context
    /// during game load.
    #[serde(skip)]
    pub egui_texture_id: bevy_egui::egui::TextureId,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TextureAtlasMeta {
    pub image: String,
    pub tile_size: Vec2,
    pub columns: usize,
    pub rows: usize,
    #[serde(default)]
    pub padding: Vec2,
    #[serde(default)]
    pub offset: Vec2,
}
