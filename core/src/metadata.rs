//! Core game metadata

use serde::Deserializer;

use crate::prelude::*;

mod common;
mod element;
mod map;
mod player;

pub use common::*;
pub use element::*;
pub use map::*;
pub use player::*;

/// Resource containing the session's [`CoreMeta`].
///
/// This is wrapped in an [`Arc`] because it doesn't change over the course of a game, and because
/// it makes it very cheap to clone during game snapshots.
#[derive(::bevy::prelude::Resource, Deref, DerefMut, TypeUlid, Clone, Default, Debug)]
#[ulid = "01GNFXQXM8FCTD1JPDTJ796A25"]
pub struct CoreMetaArc(pub Arc<CoreMeta>);

pub struct JumpyCoreAssetsPlugin;
impl ::bevy::prelude::Plugin for JumpyCoreAssetsPlugin {
    fn build(&self, app: &mut ::bevy::prelude::App) {
        use bones_bevy_asset::BonesBevyAssetAppExt;
        app.add_bones_asset::<CoreMeta>()
            .add_bones_asset::<PlayerMeta>()
            .add_bones_asset::<MapMeta>()
            .add_bones_asset::<ElementMeta>()
            .add_bones_asset::<BulletMeta>();
    }
}

#[derive(BonesBevyAsset, Clone, Debug, Deserialize, TypeUlid, Default)]
#[asset_id = "core"]
#[ulid = "01GNWT2Q8EZ5CEV3MHWNMGEEA6"]
#[serde(deny_unknown_fields)]
pub struct CoreMeta {
    pub camera: CameraMeta,
    pub physics: PhysicsMeta,
    pub players: Vec<Handle<PlayerMeta>>,
    pub stable_maps: Vec<Handle<MapMeta>>,
    pub experimental_maps: Vec<Handle<MapMeta>>,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[serde(default)]
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

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct PhysicsMeta {
    pub gravity: f32,
    pub terminal_velocity: f32,
    pub friction_lerp: f32,
    pub stop_threshold: f32,
}
