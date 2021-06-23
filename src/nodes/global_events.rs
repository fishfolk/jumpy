use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, Handle, RefMut},
    },
    prelude::*,
};

use crate::{
    nodes::{pickup::ItemType, Pickup, Player},
    Resources,
};

pub struct GlobalEvents {
    last_spawn_time: f64,
    _player: Handle<Player>,

    uid: usize,
}

impl GlobalEvents {
    const SPAWN_INTERVAL: f32 = 2.0;

    pub fn new(player: Handle<Player>, player2: Handle<Player>) -> GlobalEvents {
        GlobalEvents {
            _player: player,
            last_spawn_time: 0.0,
            uid: 0,
        }
    }
}

impl scene::Node for GlobalEvents {
    fn update(mut node: RefMut<Self>) {
        let spawned_items_len = scene::find_nodes_by_type::<Pickup>().count();
        let spawned_items = scene::find_nodes_by_type::<Pickup>();
        if get_time() - node.last_spawn_time >= Self::SPAWN_INTERVAL as _ && spawned_items_len < 3 {
            let resources = storage::get::<Resources>();

            let tilewidth = resources.tiled_map.raw_tiled_map.tilewidth as f32;
            let w = resources.tiled_map.raw_tiled_map.width as f32;
            let tileheight = resources.tiled_map.raw_tiled_map.tileheight as f32;
            let h = resources.tiled_map.raw_tiled_map.height as f32;

            let pos = loop {
                let x = rand::gen_range(0, w as i32) as f32;
                let y = rand::gen_range(0, h as i32 - 6) as f32;

                let pos = vec2((x + 0.5) * tilewidth, (y - 0.5) * tileheight);

                if resources
                    .collision_world
                    .collide_solids(pos, tilewidth as _, tileheight as _)
                    == false
                    && resources.collision_world.collide_solids(
                        pos,
                        tilewidth as _,
                        tileheight as i32 * 3,
                    )
                    && Rect::new(5. * 32., 12. * 32., 8. * 32., 6. * 32.).contains(pos) == false
                {
                    break pos;
                }
            };

            node.last_spawn_time = get_time();

            let item_type = if rand::gen_range(0, 2) == 0 {
                ItemType::Gun
            } else {
                ItemType::Sword
            };

            scene::add_node(Pickup::new(pos, item_type));

            node.uid += 1;
        }
    }
}
