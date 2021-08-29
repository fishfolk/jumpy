use macroquad::{
    experimental::{collections::storage, scene},
    math::{vec2, Vec2},
    rand,
    time::get_frame_time,
};
use macroquad_platformer::Tile;

use crate::Resources;

pub struct Bullet {
    pub pos: Vec2,
    pub speed: Vec2,
    pub lived: f32,
    pub lifetime: f32,
    pub spread: f32,
}

impl Bullet {
    pub fn new(pos: Vec2, lifetime: f32, facing: bool, speed: f32, spread: f32) -> Bullet {
        let y = rand::gen_range(-spread, spread);

        let dir = if facing { vec2(1.0, y) } else { vec2(-1.0, y) };

        Bullet {
            pos,
            speed: dir * speed,
            lived: 0.,
            lifetime,
            spread,
        }
    }

    pub fn update(&mut self) -> bool {
        self.pos += self.speed * get_frame_time();
        self.lived += get_frame_time();

        if self.lived > self.lifetime {
            return false;
        }

        for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
            if player.get_hitbox().contains(self.pos) {
                let direction = self.pos.x > (player.body.pos.x + 10.);

                scene::find_node_by_type::<crate::nodes::Camera>()
                    .unwrap()
                    .shake();

                {
                    let mut resources = storage::get_mut::<Resources>();
                    resources.hit_fxses.spawn(self.pos);
                }

                player.kill(direction);

                return false;
            }
        }

        let mut resources = storage::get_mut::<Resources>();
        if resources.collision_world.collide_solids(self.pos, 5, 5) == Tile::Solid {
            resources.hit_fxses.spawn(self.pos);
            return false;
        }

        true
    }
}
