use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use crate::components::{Animation, AnimationParams};
use crate::json;
use crate::player::Player;

/// This is used in stead of `AnimationParams`, as we have different data requirements, in the case
/// of a player character, compared to most other use cases. We want to have a default animation
/// set, for instance, that corresponds with the way the core game characters are animated, but
/// still have the possibility to declare custom animation sets, as well as have variation in size,
///
/// Refer to `crate::components::animation_player::AnimationParams` for detailed documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAnimationParams {
    #[serde(rename = "texture")]
    pub texture_id: String,
    #[serde(default = "json::default_scale")]
    pub scale: f32,
    #[serde(default, with = "json::vec2_def")]
    pub offset: Vec2,
    #[serde(default, with = "json::vec2_opt")]
    pub pivot: Option<Vec2>,
    #[serde(
        default,
        with = "json::uvec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub frame_size: Option<UVec2>,
    #[serde(
        default,
        with = "json::color_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub tint: Option<Color>,
    #[serde(default)]
    pub animations: PlayerAnimations,
}

impl From<PlayerAnimationParams> for AnimationParams {
    fn from(other: PlayerAnimationParams) -> Self {
        AnimationParams {
            texture_id: other.texture_id,
            scale: other.scale,
            offset: other.offset,
            pivot: other.pivot,
            frame_size: other.frame_size,
            tint: other.tint,
            animations: other.animations.into(),
            should_autoplay: true,
            is_deactivated: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAnimations {
    #[serde(default = "PlayerAnimations::default_idle_animation")]
    pub idle: Animation,
    #[serde(rename = "move", default = "PlayerAnimations::default_move_animation")]
    pub moving: Animation,
    #[serde(default = "PlayerAnimations::default_death_animation")]
    pub death: Animation,
    #[serde(default = "PlayerAnimations::default_death_alt_animation")]
    pub death_alt: Animation,
    #[serde(default = "PlayerAnimations::default_float_animation")]
    pub float: Animation,
    #[serde(default = "PlayerAnimations::default_crouch_animation")]
    pub crouch: Animation,
    #[serde(default = "PlayerAnimations::default_slide_animation")]
    pub slide: Animation,
}

impl PlayerAnimations {
    pub fn default_idle_animation() -> Animation {
        Animation {
            id: Player::IDLE_ANIMATION_ID.to_string(),
            row: 0,
            frames: 7,
            fps: 12,
            is_looping: true,
        }
    }

    pub fn default_move_animation() -> Animation {
        Animation {
            id: Player::MOVE_ANIMATION_ID.to_string(),
            row: 2,
            frames: 6,
            fps: 10,
            is_looping: true,
        }
    }

    pub fn default_death_animation() -> Animation {
        Animation {
            id: Player::DEATH_ANIMATION_ID.to_string(),
            row: 12,
            frames: 3,
            fps: 5,
            is_looping: false,
        }
    }

    pub fn default_death_alt_animation() -> Animation {
        Animation {
            id: Player::DEATH_ALT_ANIMATION_ID.to_string(),
            row: 14,
            frames: 4,
            fps: 8,
            is_looping: true,
        }
    }

    pub fn default_float_animation() -> Animation {
        Animation {
            id: Player::FLOAT_ANIMATION_ID.to_string(),
            row: 6,
            frames: 4,
            fps: 8,
            is_looping: true,
        }
    }

    pub fn default_crouch_animation() -> Animation {
        Animation {
            id: Player::CROUCH_ANIMATION_ID.to_string(),
            row: 16,
            frames: 2,
            fps: 8,
            is_looping: false,
        }
    }

    pub fn default_slide_animation() -> Animation {
        Animation {
            id: Player::SLIDE_ANIMATION_ID.to_string(),
            row: 18,
            frames: 2,
            fps: 8,
            is_looping: false,
        }
    }
}

impl Default for PlayerAnimations {
    fn default() -> Self {
        PlayerAnimations {
            idle: Self::default_idle_animation(),
            moving: Self::default_move_animation(),
            death: Self::default_death_animation(),
            death_alt: Self::default_death_alt_animation(),
            float: Self::default_float_animation(),
            crouch: Self::default_crouch_animation(),
            slide: Self::default_slide_animation(),
        }
    }
}

impl From<Vec<Animation>> for PlayerAnimations {
    fn from(vec: Vec<Animation>) -> Self {
        PlayerAnimations {
            idle: vec
                .iter()
                .find(|anim| anim.id == Player::IDLE_ANIMATION_ID)
                .cloned()
                .unwrap(),
            moving: vec
                .iter()
                .find(|anim| anim.id == Player::MOVE_ANIMATION_ID)
                .cloned()
                .unwrap(),
            death: vec
                .iter()
                .find(|anim| anim.id == Player::DEATH_ANIMATION_ID)
                .cloned()
                .unwrap(),
            death_alt: vec
                .iter()
                .find(|anim| anim.id == Player::DEATH_ALT_ANIMATION_ID)
                .cloned()
                .unwrap(),
            float: vec
                .iter()
                .find(|anim| anim.id == Player::FLOAT_ANIMATION_ID)
                .cloned()
                .unwrap(),
            crouch: vec
                .iter()
                .find(|anim| anim.id == Player::CROUCH_ANIMATION_ID)
                .cloned()
                .unwrap(),
            slide: vec
                .iter()
                .find(|anim| anim.id == Player::SLIDE_ANIMATION_ID)
                .cloned()
                .unwrap(),
        }
    }
}

impl From<PlayerAnimations> for Vec<Animation> {
    fn from(params: PlayerAnimations) -> Self {
        vec![
            params.idle,
            params.moving,
            params.death,
            params.death_alt,
            params.float,
            params.crouch,
            params.slide,
        ]
    }
}
