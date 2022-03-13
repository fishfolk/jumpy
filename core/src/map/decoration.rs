use crate::prelude::*;

use hecs::{Entity, World};

use serde::{Deserialize, Serialize};

use crate::drawables::Drawable;

use crate::prelude::*;

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
    let texture_res = get_texture(&meta.sprite.texture_id);

    let animations = meta
        .sprite
        .animations
        .clone()
        .into_iter()
        .map(|m| m.into())
        .collect::<Vec<_>>();

    world.spawn((
        Decoration::new(&meta.id),
        Transform::from(position),
        Drawable::new_animated_sprite(
            DECORATION_DRAW_ORDER,
            texture_res.texture,
            texture_res.frame_size(),
            animations.as_slice(),
            meta.sprite.clone().into(),
        ),
    ))
}
