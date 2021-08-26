use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::RefMut,
    },
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
    rot: f32,
    smoke_cooldown: f32,
}

struct Smoke {
    pos: Vec2,
    age: f32,
    rot: f32,
}

pub struct Bullets {
    bullets: Vec<Bullet>,
    sprite: AnimatedSprite,
    smoke: AnimatedSprite,
    smokes: Vec<Smoke>,
}

impl Bullets {
    pub const BULLET_SPEED: f32 = 500.0;
    pub const BULLET_LIFETIME: f32 = 0.9;
    pub const SMOKE_SPAWN_RATE: f32 = 8.0;

    pub fn new() -> Bullets {
        Bullets {
            bullets: Vec::with_capacity(200),
            smokes: Vec::with_capacity(200),
            sprite: AnimatedSprite::new(
                92,
                32,
                &[Animation {
                    name: "fly".to_string(),
                    row: 3,
                    frames: 1,
                    fps: 1,
                }],
                false,
            ),
            smoke: AnimatedSprite::new(
                32,
                32,
                &[Animation {
                    name: "anim".to_string(),
                    row: 0,
                    frames: 10,
                    fps: 15,
                }],
                false,
            ),
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
            rot: 0.0,
            smoke_cooldown: rand::gen_range(3.0, 13.0),
        });
    }
}

impl scene::Node for Bullets {
    fn draw(mut node: RefMut<Self>) {
        let sprite = node.sprite.clone(); // Ugly copy, but things are getting reworked anyways
        let resources = storage::get_mut::<Resources>();
        for i in 0..node.bullets.len() {
            {
                let bullet = &mut node.bullets[i];

                bullet.rot += 0.2;

                draw_texture_ex(
                    resources.gun,
                    bullet.pos.x - 52.0 * bullet.size * 0.25,
                    bullet.pos.y - 14.0 * bullet.size * 0.25,
                    color::WHITE,
                    DrawTextureParams {
                        source: Some(sprite.frame().source_rect),
                        dest_size: Some(sprite.frame().dest_size * bullet.size * 0.25),
                        rotation: bullet.rot,
                        pivot: Some(vec2(bullet.pos.x, bullet.pos.y)),
                        ..Default::default()
                    },
                );

                bullet.smoke_cooldown -= 1.;
            }
            if node.bullets[i].smoke_cooldown <= 0. {
                node.bullets[i].smoke_cooldown = Self::SMOKE_SPAWN_RATE;
                let pos = node.bullets[i].pos;
                node.smokes.push(Smoke {
                    pos,
                    age: 0.0,
                    rot: rand::gen_range(0.0, 360.0),
                })
            }

            /*draw_circle(
                bullet.pos.x,
                bullet.pos.y,
                bullet.size,
                Color::new(1.0, 1.0, 0.8, 1.0),
            );*/
        }

        let mut sm = node.smoke.clone();
        for s in &mut node.smokes {
            sm.set_frame(s.age as u32 % 10);
            sm.update();
            draw_texture_ex(
                resources.smoke_trail,
                s.pos.x - 32.0,
                s.pos.y - 32.0,
                color::WHITE,
                DrawTextureParams {
                    source: Some(sm.frame().source_rect),
                    dest_size: Some(sm.frame().dest_size * 2.0),
                    rotation: s.rot,
                    //pivot: Some(vec2(bullet.pos.x, bullet.pos.y)),
                    ..Default::default()
                },
            );
            s.age += 1.0;
        }

        node.smokes.retain(|s| s.age < 10.0);
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
