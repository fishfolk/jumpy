//! Modified from macroquad platformer:
//!
//! Copyright: @ 2019-2021 Fedor Logachev <not.fl3@gmail.com>
//! Licenses:
//!   - <https://github.com/not-fl3/macroquad/blob/master/LICENSE-MIT>
//!   - <https://github.com/not-fl3/macroquad/blob/d706128463f2656c9e0d1a46e48de27403a8feb7/LICENSE-APACHE>

use std::collections::HashSet;

use bytemuck::Zeroable;

use crate::prelude::*;

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct Rect {
    min: Vec2,
    max: Vec2,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        let half_size = vec2(width / 2.0, height / 2.0);
        let min = vec2(x, y) - half_size;
        let max = min + vec2(width, height);
        Self { min, max }
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    #[inline]
    pub fn left(&self) -> f32 {
        self.min.x
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.max.x
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.max.y
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.min.y
    }

    #[inline]
    pub fn top_left(&self) -> Vec2 {
        vec2(self.min.x, self.max.y)
    }

    #[inline]
    pub fn top_right(&self) -> Vec2 {
        vec2(self.max.x, self.max.y)
    }

    #[inline]
    pub fn bottom_left(&self) -> Vec2 {
        vec2(self.min.x, self.min.y)
    }

    #[inline]
    pub fn bottom_right(&self) -> Vec2 {
        vec2(self.max.x, self.min.y)
    }

    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() >= other.bottom()
            && self.bottom() <= other.top()
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.left()
            && point.x < self.right()
            && point.y < self.bottom()
            && point.y >= self.top()
    }

    pub fn min(&self) -> Vec2 {
        self.min
    }

    pub fn max(&self) -> Vec2 {
        self.max
    }
}

/// Macro to "derive" ( not really a derive macro ) SystemParam for a struct.
macro_rules! impl_system_param {
    (
        pub struct $t:ident<'a> {
            $(
                $f_name:ident: $f_ty:ty
            ),*
            $(,)?
        }
    ) => {
        pub struct $t<'a> {
            $(
                $f_name: $f_ty
            ),*
        }

        impl<'a> SystemParam for $t<'a> {
            type State = (
                $(
                    <$f_ty as SystemParam>::State
                ),*
            );
            type Param<'p> = $t<'p>;

            fn initialize(world: &mut World) {
                $(
                    <$f_ty as SystemParam>::initialize(world);
                )*
            }

            fn get_state(world: &World) -> Self::State {
                (
                    $(
                        <$f_ty as SystemParam>::get_state(world)
                    ),*
                )
            }

            fn borrow(state: &mut Self::State) -> Self::Param<'_> {
                let (
                    $(
                        $f_name
                    ),*
                ) = state;
                let (
                    $(
                        $f_name
                    ),*
                ) = (
                    $(
                        <$f_ty as SystemParam>::borrow($f_name)
                    ),*
                );

                Self::Param {
                    $(
                        $f_name
                    ),*
                }
            }
        }
    };
}

impl_system_param! {
    pub struct CollisionWorld<'a> {
        entities: Res<'a, Entities>,

        // Each collider is either an actor or a solid
        actors: CompMut<'a, Actor>,
        solids: CompMut<'a, Solid>,
        colliders: CompMut<'a, Collider>,

        tile_layers: Comp<'a, TileLayer>,
        tile_collision_tags: Comp<'a, TileCollisionTag>,
        tile_collisions: Comp<'a, TileCollision>,
    }
}

/// An actor in the physics simulation.
#[derive(Default, Clone, Copy, Debug, Pod, Zeroable, TypeUlid)]
#[ulid = "01GNF73PE03HFCE5MP8WC8ZKB6"]
#[repr(C)]
pub struct Actor;

/// A solid in the physics simulation.
#[derive(Default, Clone, Copy, Debug, Pod, Zeroable, TypeUlid)]
#[ulid = "01GNF73B5D1M7JYN0F65HMR2MW"]
#[repr(C)]
pub struct Solid;

/// A collider body in the physics simulation.
#[derive(Default, Clone, Debug, TypeUlid)]
#[ulid = "01GNF72YMMDM831S0TGAR2SWZ9"]
#[repr(C)]
pub struct Collider {
    pub pos: Vec2,
    pub width: f32,
    pub height: f32,
    pub x_remainder: f32,
    pub y_remainder: f32,
    pub collidable: bool,
    pub squished: bool,
    pub descent: bool,
    pub seen_wood: bool,
    pub squishers: HashSet<Entity>,
}

impl Collider {
    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.width, self.height)
    }
}

#[derive(Clone, Debug, TypeUlid, PartialEq, Eq, Copy)]
#[ulid = "01GNQKQGJMMF6AY8DRVY5YY4TF"]
#[repr(C)]
pub struct TileCollisionTag(u8);

impl Default for TileCollisionTag {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug, TypeUlid)]
#[ulid = "01GNF746HB9N9GE9E2KG4X7X4K"]
#[repr(transparent)]
pub struct TileCollision(pub u8);

impl TileCollision {
    pub const EMPTY: TileCollision = TileCollision(0);
    pub const SOLID: TileCollision = TileCollision(1);
    pub const JUMP_THROUGH: TileCollision = TileCollision(2);
    pub const COLLIDER: TileCollision = TileCollision(3);
}

impl TileCollision {
    fn or(self, other: TileCollision) -> TileCollision {
        match (self, other) {
            (TileCollision::EMPTY, TileCollision::EMPTY) => TileCollision::EMPTY,
            (TileCollision::JUMP_THROUGH, TileCollision::JUMP_THROUGH) => {
                TileCollision::JUMP_THROUGH
            }
            (TileCollision::JUMP_THROUGH, TileCollision::EMPTY) => TileCollision::JUMP_THROUGH,
            (TileCollision::EMPTY, TileCollision::JUMP_THROUGH) => TileCollision::JUMP_THROUGH,
            _ => TileCollision::SOLID,
        }
    }
}

impl<'a> CollisionWorld<'a> {
    #[allow(unused)]
    pub fn add_actor(&mut self, actor: Entity, pos: Vec2, width: f32, height: f32) {
        let mut descent = false;
        let mut seen_wood = false;
        let tile = self.collide_solids(pos, width, height);
        if tile == TileCollision::JUMP_THROUGH {
            descent = true;
            seen_wood = true;
        }
        self.colliders.insert(
            actor,
            Collider {
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
            },
        );
    }

    #[allow(unused)]
    pub fn add_solid(&mut self, solid: Entity, pos: Vec2, width: f32, height: f32) {
        self.colliders.insert(
            solid,
            Collider {
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
            },
        );
    }

    pub fn set_actor_position(&mut self, entity: Entity, pos: Vec2) {
        if self.actors.contains(entity) {
            if let Some(collider) = self.colliders.get_mut(entity) {
                collider.x_remainder = 0.0;
                collider.y_remainder = 0.0;
                collider.pos = pos;
            }
        }
    }

    /// Returns the collisions that one actor has with any other actors
    pub fn actor_collisions(&self, entity: Entity) -> Vec<Entity> {
        let mut collisions = Vec::new();

        if !self.actors.contains(entity) {
            return collisions;
        }
        let Some(collider) = self.colliders.get(entity) else {
            return collisions;
        };

        let rect = collider.rect();

        for (other_entity, (_actor, collider)) in
            self.entities.iter_with((&self.actors, &self.colliders))
        {
            if entity == other_entity {
                continue;
            }
            let other_rect = collider.rect();
            if rect.overlaps(&other_rect) {
                collisions.push(other_entity);
            }
        }

        collisions
    }

    #[allow(unused)]
    pub fn descent(&mut self, entity: Entity) {
        if self.actors.contains(entity) {
            let mut collider = self.colliders.get_mut(entity).unwrap();
            collider.descent = true;
        }
    }

    pub fn move_v(&mut self, entity: Entity, dy: f32) -> bool {
        assert!(self.actors.contains(entity));
        let mut collider = self.colliders.get(entity).unwrap().clone();

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
                if tile == TileCollision::JUMP_THROUGH && collider.descent {
                    collider.seen_wood = true;
                }
                // collider wants to go up and encoutered jumpthrough obstace
                if tile == TileCollision::JUMP_THROUGH && sign > 0 {
                    collider.seen_wood = true;
                    collider.descent = true;
                }
                if tile == TileCollision::EMPTY
                    || (tile == TileCollision::JUMP_THROUGH && collider.descent)
                {
                    collider.pos.y += sign as f32;
                    move_ -= sign;
                } else {
                    collider.pos.y = collider.pos.y.floor();
                    *self.colliders.get_mut(entity).unwrap() = collider;

                    return false;
                }
            }
        }

        // Final check, if we are out of woods after the move - reset wood flags
        let tile = self.collide_solids(collider.pos, collider.width, collider.height);
        if tile != TileCollision::JUMP_THROUGH {
            collider.seen_wood = false;
            collider.descent = false;
        }

        *self.colliders.get_mut(entity).unwrap() = collider;
        true
    }

    pub fn move_h(&mut self, entity: Entity, dx: f32) -> bool {
        assert!(self.actors.contains(entity));
        let mut collider = self.colliders.get(entity).unwrap().clone();
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
                if tile == TileCollision::JUMP_THROUGH {
                    collider.descent = true;
                    collider.seen_wood = true;
                }
                if tile == TileCollision::EMPTY || tile == TileCollision::JUMP_THROUGH {
                    collider.pos.x += sign as f32;
                    move_ -= sign;
                } else {
                    collider.pos.x = collider.pos.x.floor();
                    *self.colliders.get_mut(entity).unwrap() = collider;
                    return false;
                }
            }
        }
        *self.colliders.get_mut(entity).unwrap() = collider;
        true
    }

    #[allow(unused)]
    pub fn solid_move(&mut self, solid: Entity, dx: f32, dy: f32) {
        assert!(self.solids.contains(solid));
        let mut collider = self.colliders.get_mut(solid).unwrap();

        collider.x_remainder += dx;
        collider.y_remainder += dy;
        let move_x = collider.x_remainder.round() as i32;
        let move_y = collider.y_remainder.round() as i32;

        let mut riding_actors = vec![];
        let mut pushing_actors = vec![];

        let riding_rect = Rect::new(collider.pos.x, collider.pos.y - 1.0, collider.width, 1.0);
        let pushing_rect = Rect::new(
            collider.pos.x + move_x as f32,
            collider.pos.y,
            collider.width,
            collider.height,
        );

        for actor in self.entities.iter_with_bitset(self.actors.bitset()) {
            let actor_collider = self.colliders.get_mut(actor).unwrap();
            let rider_rect = Rect::new(
                actor_collider.pos.x,
                actor_collider.pos.y + actor_collider.height - 1.0,
                actor_collider.width,
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

        self.colliders.get_mut(solid).unwrap().collidable = false;
        for actor in riding_actors {
            self.move_h(actor, move_x as f32);
        }
        for actor in pushing_actors {
            let squished = !self.move_h(actor, move_x as f32);
            if squished {
                let mut collider = self.colliders.get_mut(actor).unwrap();
                collider.squished = true;
                collider.squishers.insert(solid);
            }
        }
        self.colliders.get_mut(solid).unwrap().collidable = true;

        let mut collider = self.colliders.get_mut(solid).unwrap();
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
        self.tag_at(pos, TileCollisionTag::default())
    }

    #[allow(unused)]
    pub fn tag_at(&self, pos: Vec2, tag: TileCollisionTag) -> bool {
        for (entity, tile_layer) in self.entities.iter_with(&self.tile_layers) {
            let TileLayer {
                tiles,
                grid_size,
                tile_size,
                ..
            } = tile_layer;

            let x = (pos.x / tile_size.y) as i32;
            let y = (pos.y / tile_size.x) as i32;
            let tile_entity = tile_layer.get(UVec2::new(x as _, y as _));
            if x >= 0
                && x < grid_size.x as i32
                && y >= 0
                && y < grid_size.y as i32
                && tile_entity.is_some()
            {
                let tile_tag = tile_entity
                    .map(|e| self.tile_collision_tags.get(e).copied().unwrap_or_default());
                return tile_tag == Some(tag);
            }
        }

        self.entities
            .iter_with((&self.solids, &self.colliders))
            .any(|(_, (_, collider))| collider.collidable && collider.rect().contains(pos))
    }

    pub fn collide_solids(&self, pos: Vec2, width: f32, height: f32) -> TileCollision {
        let tile = self.collide_tag(TileCollisionTag::default(), pos, width, height);
        if tile != TileCollision::EMPTY {
            return tile;
        }

        self.entities
            .iter_with((&self.solids, &self.colliders))
            .find(|(_entity, (_solid, collider))| {
                collider.collidable
                    && collider
                        .rect()
                        .overlaps(&Rect::new(pos.x, pos.y, width, height))
            })
            .map_or(TileCollision::EMPTY, |_| TileCollision::COLLIDER)
    }

    pub fn collide_tag(
        &self,
        tag: TileCollisionTag,
        pos: Vec2,
        width: f32,
        height: f32,
    ) -> TileCollision {
        for (_, tile_layer) in self.entities.iter_with(&self.tile_layers) {
            let TileLayer {
                grid_size,
                tile_size,
                ..
            } = tile_layer;

            let check = |pos: Vec2| {
                let y = (pos.y / tile_size.x) as i32;
                let x = (pos.x / tile_size.y) as i32;
                let tile_entity = tile_layer.get(UVec2::new(x as _, y as _));
                let tile_tag = tile_entity
                    .map(|e| self.tile_collision_tags.get(e).copied().unwrap_or_default());
                if x >= 0
                    && x < grid_size.x as i32
                    && y >= 0
                    && y < grid_size.y as i32
                    && tile_tag == Some(tag)
                {
                    tile_entity
                        .map(|entity| *self.tile_collisions.get(entity).unwrap())
                        .unwrap_or_default()
                } else {
                    TileCollision::EMPTY
                }
            };

            let hw = width / 2.0;
            let hh = height / 2.0;
            let tile = check(pos + vec2(-hw, -hh))
                .or(check(pos + vec2(-hw, hh)))
                .or(check(pos + vec2(hw, hh)))
                .or(check(pos + vec2(hw, -hh)));

            if tile != TileCollision::EMPTY {
                return tile;
            }

            if width as i32 > tile_size.x as i32 {
                let mut x = pos.x - hw;

                while {
                    x += tile_size.x;
                    x < pos.x + hw - 1.
                } {
                    let tile = check(vec2(x, pos.y - hh)).or(check(vec2(x, pos.y + hh)));
                    if tile != TileCollision::EMPTY {
                        return tile;
                    }
                }
            }

            if height as i32 > tile_size.y as i32 {
                let mut y = pos.y - hh;

                while {
                    y += tile_size.y;
                    y < pos.y + hh - 1.
                } {
                    let tile = check(vec2(pos.x - hw, y)).or(check(vec2(pos.x + hw, y)));
                    if tile != TileCollision::EMPTY {
                        return tile;
                    }
                }
            }
        }

        TileCollision::EMPTY
    }

    #[allow(unused)]
    pub fn squished(&self, actor: Entity) -> bool {
        assert!(self.actors.contains(actor));
        self.colliders.get(actor).unwrap().squished
    }

    pub fn actor_pos(&self, actor: Entity) -> Vec2 {
        assert!(self.actors.contains(actor));
        self.colliders.get(actor).unwrap().pos
    }

    #[allow(unused)]
    pub fn solid_pos(&self, solid: Entity) -> Vec2 {
        assert!(self.solids.contains(solid));
        self.colliders.get(solid).unwrap().pos
    }

    pub fn collide_check(&self, actor: Entity, pos: Vec2) -> bool {
        assert!(self.actors.contains(actor));
        let collider = self.colliders.get(actor).unwrap();

        let tile = self.collide_solids(pos, collider.width, collider.height);
        if collider.descent {
            tile == TileCollision::SOLID || tile == TileCollision::COLLIDER
        } else {
            tile == TileCollision::SOLID
                || tile == TileCollision::COLLIDER
                || tile == TileCollision::JUMP_THROUGH
        }
    }

    pub fn get_collider(&self, actor: Entity) -> &Collider {
        assert!(self.actors.contains(actor));
        self.colliders.get(actor).unwrap()
    }
}
