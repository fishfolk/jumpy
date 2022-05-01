use macroquad::prelude::*;

use hecs::{Entity, World};

use serde::{Deserialize, Serialize};

use crate::{AnimatedSpriteMetadata, Drawable, DrawableKind};
use core::Transform;

const DECORATION_DRAW_ORDER: u32 = 0;

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
    let sprite = meta.sprite.into();

    world.spawn((
        Decoration::new(&meta.id),
        Transform::from(position),
        Drawable {
            draw_order: DECORATION_DRAW_ORDER,
            kind: DrawableKind::AnimatedSprite(sprite),
        },
    ))
}
