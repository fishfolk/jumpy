//! This implements `PlayerCharacterParams`, which is a declaration of a playable character, loaded
//! from the `player_characters.json` file. This holds information like its name, its description,
//! which texture to use and how to animate it and should not be confused with `Player`, which is
//! the actual implementation of the player actor.

use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use crate::json;

mod animations;

pub use animations::PlayerAnimationParams;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCharacterParams {
    /// This is the id of the player character. This should be unique, or it will either overwrite
    /// or be overwritten, depending on load order, if not.
    pub id: String,
    /// This is the name of the player character, as shown in character selection
    pub name: String,
    /// This is the description for the player character, as shown in character selection
    #[serde(default)]
    pub description: String,
    /// This holds the animation and sprite parameters for the player character. This is flattened,
    /// meaning that, in JSON, you will declare the members of this struct directly in the
    /// `PlayerCharacterParams` entry.
    #[serde(flatten)]
    pub animation: PlayerAnimationParams,
    /// The size of the players collider.
    /// This should, in general, be smaller than the sprite size
    #[serde(
        default = "PlayerCharacterParams::default_collider_size",
        with = "json::vec2_def"
    )]
    pub collider_size: Vec2,
    /// This is the offset from the position of the player to where the weapon is mounted.
    /// The position of the player will, typically, be the center bottom of the sprite but this
    /// can be changed with offsets.
    #[serde(
        default = "PlayerCharacterParams::default_weapon_mount",
        with = "json::vec2_def"
    )]
    pub weapon_mount: Vec2,
    /// This is the distance from the top of the collider to where the head ends
    #[serde(default = "PlayerCharacterParams::default_head_threshold")]
    pub head_threshold: f32,
    /// This is the distance from the top of the collider to where the legs begin
    #[serde(default = "PlayerCharacterParams::default_legs_threshold")]
    pub legs_threshold: f32,
    /// This is the upwards force applied to the player character when it jumps
    #[serde(default = "PlayerCharacterParams::default_jump_force")]
    pub jump_force: f32,
    /// This is the movement speed of the player character
    #[serde(default = "PlayerCharacterParams::default_move_speed")]
    pub move_speed: f32,
    /// This is the slide speed factor of the player character
    #[serde(default = "PlayerCharacterParams::default_slide_speed_factor")]
    pub slide_speed_factor: f32,
    /// This is the slide duration of the player character
    #[serde(default = "PlayerCharacterParams::default_slide_duration")]
    pub slide_duration: f32,
    /// This is the float gravity factor of the player character
    #[serde(default = "PlayerCharacterParams::default_float_gravity_factor")]
    pub float_gravity_factor: f32,
}

impl PlayerCharacterParams {
    const DEFAULT_HEAD_THRESHOLD: f32 = 24.0;
    const DEFAULT_LEGS_THRESHOLD: f32 = 42.0;

    const DEFAULT_JUMP_FORCE: f32 = 600.0;
    const DEFAULT_MOVE_SPEED: f32 = 250.0;
    const DEFAULT_SLIDE_SPEED_FACTOR: f32 = 3.0;
    const DEFAULT_SLIDE_DURATION: f32 = 0.1;
    const DEFAULT_FLOAT_GRAVITY_FACTOR: f32 = 0.5;

    const DEFAULT_COLLIDER_WIDTH: f32 = 20.0;
    const DEFAULT_COLLIDER_HEIGHT: f32 = 54.0;

    const DEFAULT_WEAPON_MOUNT_X: f32 = -8.0;
    const DEFAULT_WEAPON_MOUNT_Y: f32 = 25.75;

    pub fn default_head_threshold() -> f32 {
        Self::DEFAULT_HEAD_THRESHOLD
    }

    pub fn default_legs_threshold() -> f32 {
        Self::DEFAULT_LEGS_THRESHOLD
    }

    pub fn default_jump_force() -> f32 {
        Self::DEFAULT_JUMP_FORCE
    }

    pub fn default_move_speed() -> f32 {
        Self::DEFAULT_MOVE_SPEED
    }

    pub fn default_slide_speed_factor() -> f32 {
        Self::DEFAULT_SLIDE_SPEED_FACTOR
    }

    pub fn default_slide_duration() -> f32 {
        Self::DEFAULT_SLIDE_DURATION
    }

    pub fn default_float_gravity_factor() -> f32 {
        Self::DEFAULT_FLOAT_GRAVITY_FACTOR
    }

    pub fn default_collider_size() -> Vec2 {
        vec2(Self::DEFAULT_COLLIDER_WIDTH, Self::DEFAULT_COLLIDER_HEIGHT)
    }

    pub fn default_weapon_mount() -> Vec2 {
        vec2(Self::DEFAULT_WEAPON_MOUNT_X, Self::DEFAULT_WEAPON_MOUNT_Y)
    }
}
