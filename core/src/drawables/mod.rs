mod animated_sprite;
mod sprite;

pub use animated_sprite::*;
pub use sprite::*;

use std::borrow::{Borrow, BorrowMut};

use crate::math::Size;
use hecs::World;

use crate::storage;
use crate::texture::Texture2D;
use crate::transform::Transform;
use crate::Result;

/// This is a wrapper type for all the different types of drawable sprites, used so that we can
/// access them all in one query and draw them, ordered, in one pass, according to `draw_order`.
pub struct Drawable {
    /// This is used to specify draw order on a sprite
    /// This will be used, primarily, by `Player` to draw equipped items in the right order, relative
    /// to its own sprite. This is done by multiplying the player id by ten and adding whatever offset
    /// is required to this number, to order it relative to other sprites controlled by this specific
    /// `Player` component.
    pub draw_order: u32,
    pub kind: DrawableKind,
}

impl Drawable {
    pub fn new_sprite(draw_order: u32, texture: Texture2D, params: SpriteParams) -> Self {
        let sprite = Sprite::new(texture, params);

        Drawable {
            draw_order,
            kind: DrawableKind::Sprite(sprite),
        }
    }

    pub fn new_sprite_set(draw_order: u32, sprites: &[(&str, Sprite)]) -> Self {
        let sprite_set = SpriteSet::from(sprites);

        Drawable {
            draw_order,
            kind: DrawableKind::SpriteSet(sprite_set),
        }
    }

    pub fn new_animated_sprite(
        draw_order: u32,
        texture: Texture2D,
        frame_size: Size<f32>,
        animations: &[Animation],
        params: AnimatedSpriteParams,
    ) -> Self {
        let sprite = AnimatedSprite::new(texture, frame_size, animations, params);

        Drawable {
            draw_order,
            kind: DrawableKind::AnimatedSprite(sprite),
        }
    }

    pub fn new_animated_sprite_set(draw_order: u32, sprites: &[(&str, AnimatedSprite)]) -> Self {
        let sprite_set = AnimatedSpriteSet::from(sprites);

        Drawable {
            draw_order,
            kind: DrawableKind::AnimatedSpriteSet(sprite_set),
        }
    }

    pub fn get_sprite(&self) -> Option<&Sprite> {
        match self.kind.borrow() {
            DrawableKind::Sprite(sprite) => Some(sprite),
            _ => None,
        }
    }

    pub fn get_sprite_mut(&mut self) -> Option<&mut Sprite> {
        match self.kind.borrow_mut() {
            DrawableKind::Sprite(sprite) => Some(sprite),
            _ => None,
        }
    }

    pub fn get_sprite_set(&self) -> Option<&SpriteSet> {
        match self.kind.borrow() {
            DrawableKind::SpriteSet(sprite_set) => Some(sprite_set),
            _ => None,
        }
    }

    pub fn get_sprite_set_mut(&mut self) -> Option<&mut SpriteSet> {
        match self.kind.borrow_mut() {
            DrawableKind::SpriteSet(sprite_set) => Some(sprite_set),
            _ => None,
        }
    }

    pub fn get_animated_sprite(&self) -> Option<&AnimatedSprite> {
        match self.kind.borrow() {
            DrawableKind::AnimatedSprite(sprite) => Some(sprite),
            _ => None,
        }
    }

    pub fn get_animated_sprite_mut(&mut self) -> Option<&mut AnimatedSprite> {
        match self.kind.borrow_mut() {
            DrawableKind::AnimatedSprite(sprite) => Some(sprite),
            _ => None,
        }
    }

    pub fn get_animated_sprite_set(&self) -> Option<&AnimatedSpriteSet> {
        match self.kind.borrow() {
            DrawableKind::AnimatedSpriteSet(sprite_set) => Some(sprite_set),
            _ => None,
        }
    }

    pub fn get_animated_sprite_set_mut(&mut self) -> Option<&mut AnimatedSpriteSet> {
        match self.kind.borrow_mut() {
            DrawableKind::AnimatedSpriteSet(sprite_set) => Some(sprite_set),
            _ => None,
        }
    }
}

pub enum DrawableKind {
    Sprite(Sprite),
    SpriteSet(SpriteSet),
    AnimatedSprite(AnimatedSprite),
    AnimatedSpriteSet(AnimatedSpriteSet),
}

pub fn draw_drawables(world: &mut World, _delta_time: f32) -> Result<()> {
    let mut ordered = world
        .query_mut::<&Drawable>()
        .into_iter()
        .map(|(e, drawable)| (e, drawable.draw_order))
        .collect::<Vec<_>>();

    ordered.sort_by(|&(_, a), &(_, b)| a.cmp(&b));

    for e in ordered.into_iter().map(|(e, _)| e) {
        let transform = world.get_mut::<Transform>(e).unwrap();
        let mut drawable = world.get_mut::<Drawable>(e).unwrap();

        match drawable.kind.borrow_mut() {
            DrawableKind::Sprite(sprite) => {
                draw_one_sprite(&transform, sprite);
            }
            DrawableKind::SpriteSet(sprite_set) => {
                for id in sprite_set.draw_order.iter() {
                    let sprite = sprite_set.map.get(id).unwrap();
                    draw_one_sprite(&transform, sprite);
                }
            }
            DrawableKind::AnimatedSprite(sprite) => {
                draw_one_animated_sprite(&transform, sprite);
            }
            DrawableKind::AnimatedSpriteSet(sprite_set) => {
                for id in sprite_set.draw_order.iter() {
                    let sprite = sprite_set.map.get(id).unwrap();
                    draw_one_animated_sprite(&transform, sprite);
                }
            }
        }
    }

    Ok(())
}

pub fn debug_draw_drawables(world: &mut World, _delta_time: f32) -> Result<()> {
    let mut ordered = world
        .query_mut::<&Drawable>()
        .into_iter()
        .map(|(e, drawable)| (e, drawable.draw_order))
        .collect::<Vec<_>>();

    ordered.sort_by(|&(_, a), &(_, b)| a.cmp(&b));

    for e in ordered.into_iter().map(|(e, _)| e) {
        let position = world.get_mut::<Transform>(e).map(|t| t.position).unwrap();

        let drawable = world.get_mut::<Drawable>(e).unwrap();

        match drawable.kind.borrow() {
            DrawableKind::Sprite(sprite) => {
                debug_draw_one_sprite(position, sprite);
            }
            DrawableKind::SpriteSet(sprite_set) => {
                for id in sprite_set.draw_order.iter() {
                    let sprite = sprite_set.map.get(id).unwrap();
                    debug_draw_one_sprite(position, sprite);
                }
            }
            DrawableKind::AnimatedSprite(sprite) => {
                debug_draw_one_animated_sprite(position, sprite);
            }
            DrawableKind::AnimatedSpriteSet(sprite_set) => {
                for id in sprite_set.draw_order.iter() {
                    let sprite = sprite_set.map.get(id).unwrap();
                    debug_draw_one_animated_sprite(position, sprite);
                }
            }
        }
    }

    Ok(())
}
