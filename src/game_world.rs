use macroquad_platformer::{Tile, World as CollisionWorld};

use crate::Map;

pub struct GameWorld {
    pub map: Map,
    pub collision_world: CollisionWorld,
}

impl GameWorld {
    pub fn new(map: Map) -> Self {
        let tile_cnt = (map.grid_size.x * map.grid_size.y) as usize;
        let mut static_colliders = Vec::with_capacity(tile_cnt);
        for _ in 0..tile_cnt {
            static_colliders.push(Tile::Empty);
        }

        for layer_id in &map.draw_order {
            let layer = map.layers.get(layer_id).unwrap();
            if layer.has_collision {
                for (i, (_, _, tile)) in map.get_tiles(layer_id, None).enumerate() {
                    if let Some(tile) = tile {
                        if tile
                            .attributes
                            .contains(&Map::PLATFORM_TILE_ATTRIBUTE.to_string())
                        {
                            static_colliders[i] = Tile::JumpThrough;
                        } else {
                            static_colliders[i] = Tile::Solid;
                        }
                    }
                }
            }
        }

        let mut collision_world = CollisionWorld::new();
        collision_world.add_static_tiled_layer(
            static_colliders,
            map.tile_size.x,
            map.tile_size.y,
            map.grid_size.x as usize,
            1,
        );

        GameWorld {
            map,
            collision_world,
        }
    }
}
