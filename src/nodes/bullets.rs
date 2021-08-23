use macroquad::{
    experimental::{collections::storage, scene::RefMut},
    prelude::*,
};

use crate::Resources;
use macroquad_platformer::Tile;

struct Bullet {
    pos: Vec2,
    speed: Vec2,
    size: f32,
    lived: f32,
    lifetime: f32,
}

pub struct Bullets {
    bullets: Vec<Bullet>,
}

impl Bullets {
    pub const BULLET_SPEED: f32 = 500.0;
    pub const BULLET_LIFETIME: f32 = 0.9;

    pub fn new() -> Bullets {
        Bullets {
            bullets: Vec::with_capacity(200),
        }
    }

    pub fn spawn_bullet(&mut self, pos: Vec2, size: f32, facing: bool, spread: f32) {
        let y = rand::gen_range(-spread, spread);
        let dir = if facing { vec2(1.0, y) } else { vec2(-1.0, y) };
        self.bullets.push(Bullet {
            pos: pos + vec2(16.0, 30.0) + dir * 32.0,
            speed: dir * Self::BULLET_SPEED,
            size,
            lived: 0.0,
            lifetime: Self::BULLET_LIFETIME,
        });
    }
}

impl scene::Node for Bullets {
    fn draw(node: RefMut<Self>) {
        for bullet in &node.bullets {
            draw_circle(
                bullet.pos.x,
                bullet.pos.y,
                bullet.size,
                Color::new(1.0, 1.0, 0.8, 1.0),
            );
        }
    }

    fn fixed_update(mut node: RefMut<Self>) {
        let mut resources = storage::get_mut::<Resources>();

        for bullet in &mut node.bullets {
            bullet.pos += bullet.speed * get_frame_time();
            bullet.lived += get_frame_time();
        }

        node.bullets.retain(|bullet| {
            let mut killed = false;
            for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                let self_damaged = player.get_hitbox().contains(bullet.pos);
                let direction = bullet.pos.x > (player.body.pos.x + 10.);

                if self_damaged {
                    killed = true;

                    scene::find_node_by_type::<crate::nodes::Camera>()
                        .unwrap()
                        .shake_noise(0.8, 5, 1.);

                    player.kill(direction);
                }
            }

            if resources.collision_world.collide_solids(
                bullet.pos,
                bullet.size as i32,
                bullet.size as i32,
            ) == Tile::Solid
                || killed
            {
                resources.hit_fxses.spawn(bullet.pos);
                return false;
            }
            bullet.lived < bullet.lifetime
        });
    }
}
