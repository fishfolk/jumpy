use macroquad::{
    experimental::{
        animation::{
            AnimatedSprite,
            Animation,
        },
    },
    prelude::*,
};

use crate::{
    json,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationParams {
    pub texture_id: String,
    #[serde(with = "json::def_uvec2")]
    pub tile_size: UVec2,
    #[serde(with = "json::vec_animation")]
    pub animations: Vec<Animation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteParams {
    pub texture_id: String,
    #[serde(with = "json::opt_rect")]
    pub sprite_rect: Option<Rect>,
}

pub struct Sprite {

}