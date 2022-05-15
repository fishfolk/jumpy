//! This implements `CharacterMetadata`, which is a declaration of a playable character, loaded
//! from the `player_characters.json` file. This holds information like its name, its description,
//! which texture to use and how to animate it and should not be confused with `Player`, which is
//! the actual implementation of the player actor.

use serde::{Deserialize, Serialize};

use ff_core::prelude::*;

use crate::player::PlayerAnimationMetadata;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
#[resource(name = "character", iter_only = true, crate_name = "ff_core")]
pub struct CharacterMetadata {
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
    /// `CharacterMetadata` entry.
    #[serde(flatten, alias = "animation")]
    pub sprite: PlayerAnimationMetadata,
    /// The size of the players collider.
    /// This should, in general, be smaller than the sprite size
    #[serde(default = "CharacterMetadata::default_collider_size")]
    pub collider_size: Size<f32>,
    /// This is the offset from the position of the player to where the weapon is mounted.
    /// The position of the player will, typically, be the center bottom of the sprite but this
    /// can be changed with offsets.
    #[serde(
        default = "CharacterMetadata::default_weapon_mount",
        with = "ff_core::parsing::vec2_def"
    )]
    pub weapon_mount: Vec2,
    /// This is the offset from the position of the player to where items are mounted
    #[serde(
        default = "CharacterMetadata::default_item_mount",
        with = "ff_core::parsing::vec2_def"
    )]
    pub item_mount: Vec2,
    /// This is the offset from the position of the player to where the hat is mounted
    #[serde(
        default = "CharacterMetadata::default_hat_mount",
        with = "ff_core::parsing::vec2_def"
    )]
    pub hat_mount: Vec2,
    /// This is the distance from the top of the collider to where the head ends
    #[serde(default = "CharacterMetadata::default_head_threshold")]
    pub head_threshold: f32,
    /// This is the distance from the top of the collider to where the legs begin
    #[serde(default = "CharacterMetadata::default_legs_threshold")]
    pub legs_threshold: f32,
    /// This is the upwards force applied to the player character when it jumps
    #[serde(default = "CharacterMetadata::default_jump_force")]
    pub jump_force: f32,
    /// This is the movement speed of the player character
    #[serde(default = "CharacterMetadata::default_move_speed")]
    pub move_speed: f32,
    /// This is the slide speed factor of the player character
    #[serde(default = "CharacterMetadata::default_slide_speed_factor")]
    pub slide_speed_factor: f32,
    /// This is the slide duration of the player character
    #[serde(default = "CharacterMetadata::default_slide_duration")]
    pub slide_duration: f32,
    /// This is the amount of time this character will stay incapacitated
    #[serde(default = "CharacterMetadata::default_incapacitation_duration")]
    pub incapacitation_duration: f32,
    /// This is the float gravity factor of the player character
    #[serde(default = "CharacterMetadata::default_float_gravity_factor")]
    pub float_gravity_factor: f32,
    /// This is the gravity of the player character
    #[serde(default = "CharacterMetadata::default_gravity")]
    pub gravity: f32,
}

impl CharacterMetadata {
    const DEFAULT_HEAD_THRESHOLD: f32 = 24.0;
    const DEFAULT_LEGS_THRESHOLD: f32 = 42.0;

    const DEFAULT_GRAVITY: f32 = 1.0;

    const DEFAULT_JUMP_FORCE: f32 = 9.5;
    const DEFAULT_MOVE_SPEED: f32 = 5.0;
    const DEFAULT_SLIDE_SPEED_FACTOR: f32 = 3.0;
    const DEFAULT_SLIDE_DURATION: f32 = 0.1;
    const DEFAULT_INCAPACITATION_DURATION: f32 = 3.5;
    const DEFAULT_FLOAT_GRAVITY_FACTOR: f32 = 0.5;

    const DEFAULT_COLLIDER_WIDTH: f32 = 20.0;
    const DEFAULT_COLLIDER_HEIGHT: f32 = 54.0;

    const DEFAULT_WEAPON_MOUNT_X: f32 = 0.0;
    const DEFAULT_WEAPON_MOUNT_Y: f32 = 26.0;
    const DEFAULT_ITEM_MOUNT_X: f32 = 0.0;
    const DEFAULT_ITEM_MOUNT_Y: f32 = 0.0;
    const DEFAULT_HAT_MOUNT_X: f32 = 0.0;
    const DEFAULT_HAT_MOUNT_Y: f32 = -54.0;

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

    pub fn default_incapacitation_duration() -> f32 {
        Self::DEFAULT_INCAPACITATION_DURATION
    }

    pub fn default_float_gravity_factor() -> f32 {
        Self::DEFAULT_FLOAT_GRAVITY_FACTOR
    }

    pub fn default_collider_size() -> Size<f32> {
        Size::new(Self::DEFAULT_COLLIDER_WIDTH, Self::DEFAULT_COLLIDER_HEIGHT)
    }

    pub fn default_weapon_mount() -> Vec2 {
        vec2(Self::DEFAULT_WEAPON_MOUNT_X, Self::DEFAULT_WEAPON_MOUNT_Y)
    }

    pub fn default_item_mount() -> Vec2 {
        vec2(Self::DEFAULT_ITEM_MOUNT_X, Self::DEFAULT_ITEM_MOUNT_Y)
    }

    pub fn default_hat_mount() -> Vec2 {
        vec2(Self::DEFAULT_HAT_MOUNT_X, Self::DEFAULT_HAT_MOUNT_Y)
    }

    pub fn default_gravity() -> f32 {
        Self::DEFAULT_GRAVITY
    }
}
