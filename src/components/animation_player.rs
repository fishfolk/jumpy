use macroquad::{experimental::animation::Animation, prelude::*};

use serde::{Deserialize, Serialize};

use crate::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationParams {
    pub texture_id: String,
    #[serde(
        default,
        with = "json::vec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub offset: Option<Vec2>,
    #[serde(with = "json::uvec2_def")]
    pub tile_size: UVec2,
    #[serde(with = "json::animation_vec")]
    pub animations: Vec<Animation>,
    pub should_autoplay: bool,
}

pub struct AnimationPlayer {}
