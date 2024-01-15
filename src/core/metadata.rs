//! Core game metadata

use serde::Deserializer;
use std::time::Duration;

use crate::prelude::*;

mod map;
mod player;

pub use map::*;
pub use player::*;

/// Extension trait for the bones [`AssetServer`].
pub trait MatchAssetServerExt {
    /// Register the default assets from `bones_framework`.
    fn register_match_assets(self) -> Self;
}
impl MatchAssetServerExt for &mut AssetServer {
    fn register_match_assets(self) -> Self {
        CoreMeta::register_schema();
        PlayerMeta::register_schema();
        ElementMeta::register_schema();
        BulletMeta::register_schema();
        MapMeta::register_schema();
        HatMeta::register_schema();

        self
    }
}

#[derive(Clone, Debug, HasSchema, Default)]
#[repr(C)]
pub struct CoreMeta {
    pub camera: CameraMeta,
    pub physics: PhysicsMeta,
    pub config: CoreConfigMeta,
    pub map_tilesets: SVec<Handle<Atlas>>,
    pub players: SVec<Handle<PlayerMeta>>,
    pub player_emotes: SMap<Ustr, Handle<EmoteMeta>>,
    pub player_hats: SVec<Handle<HatMeta>>,
    pub stable_maps: SVec<Handle<MapMeta>>,
    pub map_elements: SVec<Handle<ElementMeta>>,
    pub experimental_maps: SVec<Handle<MapMeta>>,
}

#[derive(HasSchema, Clone, Debug)]
#[repr(C)]
pub struct CameraMeta {
    pub default_height: f32,
    pub border_top: f32,
    pub border_bottom: f32,
    pub border_left: f32,
    pub border_right: f32,
    pub move_lerp_factor: f32,
    pub zoom_in_lerp_factor: f32,
    pub zoom_out_lerp_factor: f32,
    pub min_camera_size: Vec2,
    pub player_camera_box_size: Vec2,
}

impl Default for CameraMeta {
    fn default() -> Self {
        Self {
            default_height: 400.0,
            border_top: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            border_right: 0.0,
            move_lerp_factor: 1.0,
            zoom_in_lerp_factor: 1.0,
            zoom_out_lerp_factor: 1.0,
            min_camera_size: Vec2::ZERO,
            player_camera_box_size: Vec2::ZERO,
        }
    }
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct PhysicsMeta {
    pub gravity: f32,
    pub terminal_velocity: f32,
    pub friction_lerp: f32,
    pub stop_threshold: f32,
}

#[derive(HasSchema, Deserialize, Clone, Debug, Default)]
#[derive_type_data(SchemaDeserialize)]
pub struct CoreConfigMeta {
    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub respawn_invincibility_time: Duration,
}
