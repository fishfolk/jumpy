use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use hecs::{Entity, World};

use serde::{Deserialize, Serialize};

use crate::{AnimatedSprite, AnimatedSpriteMetadata};
use crate::{Resources, Transform};

#[derive(Clone, Serialize, Deserialize)]
pub struct DecorationMetadata {
    pub id: String,
    pub sprite: AnimatedSpriteMetadata,
}

pub struct Decoration {
    pub id: String,
}

impl Decoration {
    pub fn new(id: &str) -> Self {
        Decoration { id: id.to_string() }
    }
}

pub fn spawn_decoration(world: &mut World, position: Vec2, meta: DecorationMetadata) -> Entity {
    let (texture, frame_size) = storage::get::<Resources>()
        .textures
        .get(&meta.sprite.texture_id)
        .map(|t| (t.texture, t.frame_size()))
        .unwrap();

    let animations = meta
        .sprite
        .animations
        .clone()
        .into_iter()
        .map(|m| m.into())
        .collect::<Vec<_>>();

    let params = meta.sprite.into();

    world.spawn((
        Decoration::new(&meta.id),
        Transform::from(position),
        AnimatedSprite::new(texture, frame_size, animations.as_slice(), params),
    ))
}
