//! Modified from macroquad platformer:
//!
//! Copyright: @ 2019-2021 Fedor Logachev <not.fl3@gmail.com>
//! Licenses:
//!   - <https://github.com/not-fl3/macroquad/blob/master/LICENSE-MIT>
//!   - <https://github.com/not-fl3/macroquad/blob/d706128463f2656c9e0d1a46e48de27403a8feb7/LICENSE-APACHE>

use bevy::{ecs::system::SystemParam, math::vec2};
use bevy_ecs_tilemap::{
    prelude::{TilemapGridSize, TilemapTileSize},
    tiles::TileStorage,
};

use crate::prelude::*;

use std::collections::HashSet;

struct Rect {
    min: Vec2,
    max: Vec2,
}

impl Rect {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Rect {
        let half_size = vec2(width / 2.0, height / 2.0);
        let min = vec2(x, y) - half_size;
        let max = min + half_size;
        Rect { min, max }
    }

    #[inline]
    fn left(&self) -> f32 {
        self.min.x
    }

    #[inline]
    fn right(&self) -> f32 {
        self.max.x
    }

    #[inline]
    fn top(&self) -> f32 {
        self.max.y
    }

    #[inline]
    fn bottom(&self) -> f32 {
        self.min.y
    }

    fn overlaps(&self, other: &Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }

    fn contains(&self, point: Vec2) -> bool {
        point.x >= self.left()
            && point.x < self.right()
            && point.y < self.bottom()
            && point.y >= self.top()
    }
}

#[derive(Component, DerefMut, Deref)]
pub struct CollisionLayerTag(pub u8);

impl Default for CollisionLayerTag {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(SystemParam)]
pub struct CollisionWorld<'w, 's> {
    commands: Commands<'w, 's>,
    static_tiled_layers: Query<
        'w,
        's,
        (
            &'static mut TileStorage,
            &'static TilemapGridSize,
            &'static TilemapTileSize,
            &'static CollisionLayerTag,
        ),
    >,
    tile_collisions: Query<'w, 's, &'static mut TileCollision>,
    solids: Query<'w, 's, &'static mut Collider, (With<Solid>, Without<Actor>)>,
    actors: Query<'w, 's, (Entity, &'static mut Collider), With<Actor>>,
}

#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component, Default)]
pub struct Actor;

#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component, Default)]
pub struct Solid;

#[derive(Component, Default, Clone, Debug)]
pub struct Collider {
    pub collidable: bool,
    pub squished: bool,
    pub pos: Vec2,
    pub width: f32,
    pub height: f32,
    pub x_remainder: f32,
    pub y_remainder: f32,
    pub squishers: HashSet<Entity>,
    pub descent: bool,
    pub seen_wood: bool,
}

impl Collider {
    fn rect(&self) -> Rect {
        Rect::new(
            self.pos.x,
            self.pos.y,
            self.width as f32,
            self.height as f32,
        )
    }
}

// #[derive(Component, Reflect, Default, Clone, Debug)]
// #[reflect(Component, Default)]
// pub struct StaticTiledLayer {
//     static_colliders: Vec<Entity>,
//     tile_width: f32,
//     tile_height: f32,
//     width: usize,
//     tag: u8,
// }

#[derive(Component, Reflect, Default, PartialEq, Eq, Clone, Copy, Debug)]
#[reflect_value(Component, Default, PartialEq)]
pub enum TileCollision {
    #[default]
    Empty,
    Solid,
    JumpThrough,
    Collider,
}

impl TileCollision {
    fn or(self, other: TileCollision) -> TileCollision {
        match (self, other) {
            (TileCollision::Empty, TileCollision::Empty) => TileCollision::Empty,
            (TileCollision::JumpThrough, TileCollision::JumpThrough) => TileCollision::JumpThrough,
            (TileCollision::JumpThrough, TileCollision::Empty) => TileCollision::JumpThrough,
            (TileCollision::Empty, TileCollision::JumpThrough) => TileCollision::JumpThrough,
            _ => TileCollision::Solid,
        }
    }
}

impl<'w, 's> CollisionWorld<'w, 's> {
    #[allow(unused)]
    pub fn add_actor(&mut self, actor: Entity, pos: Vec2, width: f32, height: f32) {
        let mut descent = false;
        let mut seen_wood = false;
        let tile = self.collide_solids(pos, width, height);
        if tile == TileCollision::JumpThrough {
            descent = true;
            seen_wood = true;
        }
        self.commands.entity(actor).insert(Collider {
            collidable: true,
            squished: false,
            pos,
            width,
            height,
            x_remainder: 0.,
            y_remainder: 0.,
            squishers: HashSet::new(),
            descent,
            seen_wood,
        });
    }

    #[allow(unused)]
    pub fn add_solid(&mut self, solid: Entity, pos: Vec2, width: f32, height: f32) {
        self.commands.entity(solid).insert(Collider {
            collidable: true,
            squished: false,
            pos,
            width,
            height,
            x_remainder: 0.,
            y_remainder: 0.,
            squishers: HashSet::new(),
            descent: false,
            seen_wood: false,
        });
    }

    pub fn set_actor_position(&mut self, entity: Entity, pos: Vec2) {
        let mut collider = self.actors.get_mut(entity).unwrap().1;

        collider.x_remainder = 0.0;
        collider.y_remainder = 0.0;
        collider.pos = pos;
    }

    #[allow(unused)]
    pub fn descent(&mut self, entity: Entity) {
        let mut collider = self.actors.get_mut(entity).unwrap().1;
        collider.descent = true;
    }

    pub fn move_v(&mut self, entity: Entity, dy: f32) -> bool {
        let mut collider = self.actors.get(entity).unwrap().1.clone();

        collider.y_remainder += dy;

        let mut move_ = collider.y_remainder.round() as i32;
        if move_ != 0 {
            collider.y_remainder -= move_ as f32;
            let sign = move_.signum();

            while move_ != 0 {
                let tile = self.collide_solids(
                    collider.pos + vec2(0., sign as f32),
                    collider.width,
                    collider.height,
                );

                // collider wants to go down and collided with jumpthrough tile
                if tile == TileCollision::JumpThrough && collider.descent {
                    collider.seen_wood = true;
                }
                // collider wants to go up and encoutered jumpthrough obstace
                if tile == TileCollision::JumpThrough && sign < 0 {
                    collider.seen_wood = true;
                    collider.descent = true;
                }
                if tile == TileCollision::Empty
                    || (tile == TileCollision::JumpThrough && collider.descent)
                {
                    collider.pos.y += sign as f32;
                    move_ -= sign;
                } else {
                    collider.pos.y = collider.pos.y.floor();
                    *self.actors.get_mut(entity).unwrap().1 = collider;

                    return false;
                }
            }
        }

        // Final check, if we are out of woods after the move - reset wood flags
        let tile = self.collide_solids(collider.pos, collider.width, collider.height);
        if tile != TileCollision::JumpThrough {
            collider.seen_wood = false;
            collider.descent = false;
        }

        *self.actors.get_mut(entity).unwrap().1 = collider;
        true
    }

    pub fn move_h(&mut self, entity: Entity, dx: f32) -> bool {
        let mut collider = self.actors.get(entity).unwrap().1.clone();
        collider.x_remainder += dx;

        let mut move_ = collider.x_remainder.round() as i32;
        if move_ != 0 {
            collider.x_remainder -= move_ as f32;
            let sign = move_.signum();

            while move_ != 0 {
                let tile = self.collide_solids(
                    collider.pos + vec2(sign as f32, 0.),
                    collider.width,
                    collider.height,
                );
                if tile == TileCollision::JumpThrough {
                    collider.descent = true;
                    collider.seen_wood = true;
                }
                if tile == TileCollision::Empty || tile == TileCollision::JumpThrough {
                    collider.pos.x += sign as f32;
                    move_ -= sign;
                } else {
                    collider.pos.x = collider.pos.x.floor();
                    *self.actors.get_mut(entity).unwrap().1 = collider;
                    return false;
                }
            }
        }
        *self.actors.get_mut(entity).unwrap().1 = collider;
        true
    }

    #[allow(unused)]
    pub fn solid_move(&mut self, solid: Entity, dx: f32, dy: f32) {
        let mut collider = self.solids.get_mut(solid).unwrap();

        collider.x_remainder += dx;
        collider.y_remainder += dy;
        let move_x = collider.x_remainder.round() as i32;
        let move_y = collider.y_remainder.round() as i32;

        let mut riding_actors = vec![];
        let mut pushing_actors = vec![];

        let riding_rect = Rect::new(
            collider.pos.x,
            collider.pos.y - 1.0,
            collider.width as f32,
            1.0,
        );
        let pushing_rect = Rect::new(
            collider.pos.x + move_x as f32,
            collider.pos.y,
            collider.width as f32,
            collider.height as f32,
        );

        for (actor, mut actor_collider) in &mut self.actors {
            let rider_rect = Rect::new(
                actor_collider.pos.x,
                actor_collider.pos.y + actor_collider.height as f32 - 1.0,
                actor_collider.width as f32,
                1.0,
            );

            if riding_rect.overlaps(&rider_rect) {
                riding_actors.push(actor);
            } else if pushing_rect.overlaps(&actor_collider.rect()) && !actor_collider.squished {
                pushing_actors.push(actor);
            }

            if !pushing_rect.overlaps(&actor_collider.rect()) {
                actor_collider.squishers.remove(&solid);
                if actor_collider.squishers.is_empty() {
                    actor_collider.squished = false;
                }
            }
        }

        self.solids.get_mut(solid).unwrap().collidable = false;
        for actor in riding_actors {
            self.move_h(actor, move_x as f32);
        }
        for actor in pushing_actors {
            let squished = !self.move_h(actor, move_x as f32);
            if squished {
                let mut collider = self.actors.get_mut(actor).unwrap().1;
                collider.squished = true;
                collider.squishers.insert(solid);
            }
        }
        self.solids.get_mut(solid).unwrap().collidable = true;

        let mut collider = self.solids.get_mut(solid).unwrap();
        if move_x != 0 {
            collider.x_remainder -= move_x as f32;
            collider.pos.x += move_x as f32;
        }
        if move_y != 0 {
            collider.y_remainder -= move_y as f32;
            collider.pos.y += move_y as f32;
        }
    }

    #[allow(unused)]
    pub fn solid_at(&self, pos: Vec2) -> bool {
        self.tag_at(pos, 1)
    }

    #[allow(unused)]
    pub fn tag_at(&self, pos: Vec2, tag: u8) -> bool {
        for (storage, grid_size, tile_size, layer_tag) in &self.static_tiled_layers {
            let x = (pos.x / tile_size.y) as i32;
            let y = (pos.y / tile_size.x) as i32;
            if x >= 0
                && x < grid_size.x as i32
                && y >= 0
                && y < grid_size.y as i32
                && storage.get(&UVec2::new(x as _, y as _).into()).is_some()
            {
                return **layer_tag == tag;
            }
        }

        self.solids
            .iter()
            .any(|solid| solid.collidable && solid.rect().contains(pos))
    }

    pub fn collide_solids(&self, pos: Vec2, width: f32, height: f32) -> TileCollision {
        let tile = self.collide_tag(1, pos, width, height);
        if tile != TileCollision::Empty {
            return tile;
        }

        self.solids
            .iter()
            .find(|solid| {
                solid.collidable
                    && solid
                        .rect()
                        .overlaps(&Rect::new(pos.x, pos.y, width, height))
            })
            .map_or(TileCollision::Empty, |_| TileCollision::Collider)
    }

    pub fn collide_tag(&self, tag: u8, pos: Vec2, width: f32, height: f32) -> TileCollision {
        for (storage, grid_size, tile_size, layer_tag) in &self.static_tiled_layers {
            let check = |pos: Vec2| {
                let y = (pos.y / tile_size.x) as i32;
                let x = (pos.x / tile_size.y) as i32;
                if x >= 0
                    && x < grid_size.x as i32
                    && y >= 0
                    && y < grid_size.y as i32
                    && **layer_tag == tag
                {
                    storage
                        .get(&UVec2::new(x as _, y as _).into())
                        .map(|entity| *self.tile_collisions.get(entity).unwrap())
                        .unwrap_or_default()
                } else {
                    TileCollision::Empty
                }
            };

            let hw = width / 2.0;
            let hh = height / 2.0;
            let tile = check(pos + vec2(-hw, -hh))
                .or(check(pos + vec2(-hw, hh)))
                .or(check(pos + vec2(hw, hh)))
                .or(check(pos + vec2(hw, -hh)));

            if tile != TileCollision::Empty {
                return tile;
            }

            if width as i32 > tile_size.x as i32 {
                let mut x = pos.x;

                while {
                    x += tile_size.x;
                    x < pos.x + width as f32 - 1.
                } {
                    let tile =
                        check(vec2(x, pos.y)).or(check(vec2(x, pos.y + height as f32 - 1.0)));
                    if tile != TileCollision::Empty {
                        return tile;
                    }
                }
            }

            if height as i32 > tile_size.y as i32 {
                let mut y = pos.y;

                while {
                    y += tile_size.y;
                    y < pos.y + height as f32 - 1.
                } {
                    let tile = check(vec2(pos.x, y)).or(check(vec2(pos.x + width as f32 - 1., y)));
                    if tile != TileCollision::Empty {
                        return tile;
                    }
                }
            }
        }

        TileCollision::Empty
    }

    #[allow(unused)]
    pub fn squished(&self, actor: Entity) -> bool {
        self.actors.get(actor).unwrap().1.squished
    }

    pub fn actor_pos(&self, actor: Entity) -> Vec2 {
        self.actors.get(actor).unwrap().1.pos
    }

    #[allow(unused)]
    pub fn solid_pos(&self, solid: Entity) -> Vec2 {
        self.solids.get(solid).unwrap().pos
    }

    pub fn collide_check(&self, collider: Entity, pos: Vec2) -> bool {
        let collider = self.actors.get(collider).unwrap().1;

        let tile = self.collide_solids(pos, collider.width, collider.height);
        if collider.descent {
            tile == TileCollision::Solid || tile == TileCollision::Collider
        } else {
            tile == TileCollision::Solid
                || tile == TileCollision::Collider
                || tile == TileCollision::JumpThrough
        }
    }
}
