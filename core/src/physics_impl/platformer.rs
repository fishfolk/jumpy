use std::collections::HashSet;
use std::time::Duration;

use crate::map::{Map, MapLayer};
use crate::math::{ivec2, vec2, Rect, Size, Vec2};
use crate::Result;

const DEFAULT_PHYSICS_RESOLUTION: u32 = 120;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Actor(pub(crate) usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Solid(pub(crate) usize);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ColliderKind {
    Empty,
    Solid,
    Platform,
    Collider,
}

impl ColliderKind {
    fn or(self, other: ColliderKind) -> ColliderKind {
        match (self, other) {
            (ColliderKind::Empty, ColliderKind::Empty) => ColliderKind::Empty,
            (ColliderKind::Platform, ColliderKind::Platform) => ColliderKind::Platform,
            (ColliderKind::Platform, ColliderKind::Empty) => ColliderKind::Platform,
            (ColliderKind::Empty, ColliderKind::Platform) => ColliderKind::Platform,
            _ => ColliderKind::Solid,
        }
    }
}

pub struct TileLayer {
    tiles: Vec<ColliderKind>,
    tile_size: Size<f32>,
    width: usize,
    tag: u8,
}

#[derive(Clone, Debug)]
struct Collider {
    position: Vec2,
    size: Size<f32>,
    remaining_movement: Vec2,
    squished_by: HashSet<Solid>,
    is_active: bool,
    is_descending: bool,
    is_squished: bool,
    has_seen_platform: bool,
}

impl Collider {
    pub fn rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.size.width as f32,
            self.size.height as f32,
        )
    }
}

const DEFAULT_TAG: u8 = 1;

pub struct PhysicsWorld {
    tile_layers: Vec<TileLayer>,
    solids: Vec<(Solid, Collider)>,
    actors: Vec<(Actor, Collider)>,
    resolution: u32,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new(DEFAULT_PHYSICS_RESOLUTION)
    }
}

impl PhysicsWorld {
    pub fn new(resolution: u32) -> PhysicsWorld {
        PhysicsWorld {
            tile_layers: vec![],
            actors: vec![],
            solids: vec![],
            resolution,
        }
    }

    pub fn fixed_delta_time(&self) -> Duration {
        Duration::from_secs_f32(1.0 / self.resolution as f32)
    }

    pub fn add_actor(&mut self, pos: Vec2, size: Size<f32>) -> Actor {
        let actor = Actor(self.actors.len());

        let mut is_descending = false;
        let mut has_seen_platform = false;
        let tile = self.collide_solids_at(pos, size);
        if tile == ColliderKind::Platform {
            is_descending = true;
            has_seen_platform = true;
        }

        self.actors.push((
            actor,
            Collider {
                position: pos,
                size,
                remaining_movement: Vec2::ZERO,
                squished_by: HashSet::new(),
                is_active: true,
                is_squished: false,
                is_descending,
                has_seen_platform,
            },
        ));

        actor
    }

    pub fn add_solid(&mut self, position: Vec2, size: Size<f32>) -> Solid {
        let solid = Solid(self.solids.len());

        self.solids.push((
            solid,
            Collider {
                position,
                size,
                remaining_movement: Vec2::ZERO,
                squished_by: HashSet::new(),
                is_descending: false,
                has_seen_platform: false,
                is_active: true,
                is_squished: false,
            },
        ));

        solid
    }

    pub fn set_actor_position(&mut self, actor: Actor, position: Vec2) {
        let collider = &mut self.actors[actor.0].1;

        collider.remaining_movement.x = 0.0;
        collider.remaining_movement.y = 0.0;
        collider.position = position;
    }

    pub fn descend(&mut self, actor: Actor) {
        let collider = &mut self.actors[actor.0].1;
        collider.is_descending = true;
    }

    pub fn move_actor_x(&mut self, actor: Actor, dx: f32) -> bool {
        let id = actor.0;
        let mut collider = self.actors[id].1.clone();
        collider.remaining_movement.x += dx;

        let mut move_ = collider.remaining_movement.x.round() as i32;
        if move_ != 0 {
            collider.remaining_movement.x -= move_ as f32;
            let sign = move_.signum();

            while move_ != 0 {
                let tile = self
                    .collide_solids_at(collider.position + vec2(sign as f32, 0.), collider.size);
                if tile == ColliderKind::Platform {
                    collider.is_descending = true;
                    collider.has_seen_platform = true;
                }
                if tile == ColliderKind::Empty || tile == ColliderKind::Platform {
                    collider.position.x += sign as f32;
                    move_ -= sign;
                } else {
                    self.actors[id].1 = collider;
                    return false;
                }
            }
        }

        self.actors[id].1 = collider;

        true
    }

    pub fn move_actor_y(&mut self, actor: Actor, dy: f32) -> bool {
        let id = actor.0;
        let mut collider = self.actors[id].1.clone();

        collider.remaining_movement.y += dy;

        let mut move_ = collider.remaining_movement.y.round() as i32;
        if move_ != 0 {
            collider.remaining_movement.y -= move_ as f32;
            let sign = move_.signum();

            while move_ != 0 {
                let tile = self
                    .collide_solids_at(collider.position + vec2(0., sign as f32), collider.size);

                // collider wants to go down and collided with platform tile
                if tile == ColliderKind::Platform && collider.is_descending {
                    collider.has_seen_platform = true;
                }
                // collider wants to go up and encountered platform obstacle
                if tile == ColliderKind::Platform && sign < 0 {
                    collider.has_seen_platform = true;
                    collider.is_descending = true;
                }
                if tile == ColliderKind::Empty
                    || (tile == ColliderKind::Platform && collider.is_descending)
                {
                    collider.position.y += sign as f32;
                    move_ -= sign;
                } else {
                    self.actors[id].1 = collider;

                    return false;
                }
            }
        }

        // Final check, if we are out of woods after the move - reset wood flags
        let tile = self.collide_solids_at(collider.position, collider.size);
        if tile != ColliderKind::Platform {
            collider.has_seen_platform = false;
            collider.is_descending = false;
        }

        self.actors[id].1 = collider;

        true
    }

    pub fn move_actor(&mut self, actor: Actor, movement: Vec2) -> bool {
        if self.move_actor_x(actor, movement.x) {
            self.move_actor_y(actor, movement.y)
        } else {
            false
        }
    }

    pub fn move_solid(&mut self, solid: Solid, movement: Vec2) {
        let collider = &mut self.solids[solid.0].1;

        collider.remaining_movement.x += movement.x;
        collider.remaining_movement.y += movement.y;

        let movement = ivec2(
            collider.remaining_movement.x.round() as i32,
            collider.remaining_movement.y.round() as i32,
        );

        let mut riding_actors = vec![];
        let mut pushing_actors = vec![];

        let riding_rect = Rect::new(
            collider.position.x,
            collider.position.y - 1.0,
            collider.size.width as f32,
            1.0,
        );

        let pushing_rect = Rect::new(
            collider.position.x + movement.x as f32,
            collider.position.y,
            collider.size.width as f32 - 1.0,
            collider.size.height as f32,
        );

        for (actor, actor_collider) in &mut self.actors {
            let rider_rect = Rect::new(
                actor_collider.position.x,
                actor_collider.position.y + actor_collider.size.height as f32 - 1.0,
                actor_collider.size.width as f32,
                1.0,
            );

            if riding_rect.overlaps(&rider_rect) {
                riding_actors.push(*actor);
            } else if pushing_rect.overlaps(&actor_collider.rect())
                && actor_collider.is_squished == false
            {
                pushing_actors.push(*actor);
            }

            if pushing_rect.overlaps(&actor_collider.rect()) == false {
                actor_collider.squished_by.remove(&solid);
                if actor_collider.squished_by.len() == 0 {
                    actor_collider.is_squished = false;
                }
            }
        }

        self.solids[solid.0].1.is_active = false;
        for actor in riding_actors {
            self.move_actor_x(actor, movement.x as f32);
        }

        for actor in pushing_actors {
            let squished = !self.move_actor_x(actor, movement.x as f32);
            if squished {
                self.actors[actor.0].1.is_squished = true;
                self.actors[actor.0].1.squished_by.insert(solid);
            }
        }

        self.solids[solid.0].1.is_active = true;

        let collider = &mut self.solids[solid.0].1;
        if movement.x != 0 {
            collider.remaining_movement.x -= movement.x as f32;
            collider.position.x += movement.x as f32;
        }
        if movement.y != 0 {
            collider.remaining_movement.y -= movement.y as f32;
            collider.position.y += movement.y as f32;
        }
    }

    pub fn is_solid_at(&self, position: Vec2) -> bool {
        self.is_tag_at(position, 1)
    }

    pub fn is_tag_at(&self, position: Vec2, tag: u8) -> bool {
        for layer in &self.tile_layers {
            let x = (position.x / layer.tile_size.height) as i32;
            let y = (position.y / layer.tile_size.width) as i32;

            let ix = y * (layer.width as i32) + x;

            if ix >= 0
                && ix < layer.tiles.len() as i32
                && layer.tiles[ix as usize] != ColliderKind::Empty
            {
                return layer.tag == tag;
            }
        }

        self.solids
            .iter()
            .any(|solid| solid.1.is_active && solid.1.rect().contains(position))
    }

    pub fn collide_solids_at(&self, position: Vec2, size: Size<f32>) -> ColliderKind {
        let tile = self.collide_tag_at(1, position, size);
        if tile != ColliderKind::Empty {
            return tile;
        }

        self.solids
            .iter()
            .find(|solid| {
                solid.1.is_active
                    && solid.1.rect().overlaps(&Rect::new(
                        position.x,
                        position.y,
                        size.width as f32,
                        size.height as f32,
                    ))
            })
            .map_or(ColliderKind::Empty, |_| ColliderKind::Collider)
    }

    pub fn collide_tag_at(&self, tag: u8, position: Vec2, size: Size<f32>) -> ColliderKind {
        for layer in &self.tile_layers {
            let check = |position: Vec2| {
                let y = (position.y / layer.tile_size.width) as i32;
                let x = (position.x / layer.tile_size.height) as i32;
                let ix = y * (layer.width as i32) + x;
                if ix >= 0
                    && ix < layer.tiles.len() as i32
                    && layer.tag == tag
                    && layer.tiles[ix as usize] != ColliderKind::Empty
                {
                    return layer.tiles[ix as usize];
                }
                return ColliderKind::Empty;
            };

            let tile = check(position)
                .or(check(position + vec2(size.width as f32 - 1.0, 0.0)))
                .or(check(
                    position + vec2(size.width as f32 - 1.0, size.height as f32 - 1.0),
                ))
                .or(check(position + vec2(0.0, size.height as f32 - 1.0)));

            if tile != ColliderKind::Empty {
                return tile;
            }

            if size.width > layer.tile_size.width {
                let mut x = position.x;

                while {
                    x += layer.tile_size.width;
                    x < position.x + size.width as f32 - 1.
                } {
                    let tile = check(vec2(x, position.y))
                        .or(check(vec2(x, position.y + size.height as f32 - 1.0)));
                    if tile != ColliderKind::Empty {
                        return tile;
                    }
                }
            }

            if size.height > layer.tile_size.height {
                let mut y = position.y;

                while {
                    y += layer.tile_size.height;
                    y < position.y + size.height as f32 - 1.
                } {
                    let tile = check(vec2(position.x, y))
                        .or(check(vec2(position.x + size.width as f32 - 1., y)));
                    if tile != ColliderKind::Empty {
                        return tile;
                    }
                }
            }
        }

        return ColliderKind::Empty;
    }

    pub fn is_squished(&self, actor: Actor) -> bool {
        self.actors[actor.0].1.is_squished
    }

    pub fn actor_position(&self, actor: Actor) -> Vec2 {
        self.actors[actor.0].1.position
    }

    pub fn solid_position(&self, solid: Solid) -> Vec2 {
        self.solids[solid.0].1.position
    }

    pub fn collide_at(&self, actor: Actor, position: Vec2) -> bool {
        let collider = &self.actors[actor.0];

        let tile = self.collide_solids_at(position, collider.1.size);
        if collider.1.is_descending {
            tile == ColliderKind::Solid || tile == ColliderKind::Collider
        } else {
            tile == ColliderKind::Solid
                || tile == ColliderKind::Collider
                || tile == ColliderKind::Platform
        }
    }

    pub fn add_layer(&mut self, tag: u8, tile_size: Size<f32>, layer: &MapLayer) {
        let tile_cnt = (layer.grid_size.width * layer.grid_size.height) as usize;

        let mut tiles = Vec::with_capacity(tile_cnt);
        for _ in 0..tile_cnt {
            tiles.push(ColliderKind::Empty);
        }

        for (i, tile) in layer.tiles.iter().enumerate() {
            if let Some(tile) = tile {
                if tile
                    .attributes
                    .contains(&Map::PLATFORM_TILE_ATTRIBUTE.to_string())
                {
                    tiles[i] = ColliderKind::Platform;
                } else {
                    tiles[i] = ColliderKind::Solid;
                }
            }
        }

        self.tile_layers.push(TileLayer {
            tiles,
            tile_size,
            width: layer.grid_size.width as usize,
            tag,
        });
    }

    pub fn add_map(&mut self, map: &Map) {
        for layer_id in &map.draw_order {
            let layer = map.layers.get(layer_id).unwrap();
            if layer.has_collision {
                self.add_layer(DEFAULT_TAG, map.tile_size, layer);
            }
        }
    }

    pub fn clear(&mut self) {
        self.actors.clear();
        self.solids.clear();
        self.tile_layers.clear();
    }
}
