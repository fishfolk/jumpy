use macroquad::{
    experimental::{
        collections::storage,
        scene::RefMut,
        animation::{
            AnimatedSprite,
            Animation,
        },
    },
    color,
    prelude::*,
};

use crate::{
    nodes::player::PhysicsBody,
    Resources,
};

pub struct ArmedGrenade {
    grenade_sprite: AnimatedSprite,
    body: PhysicsBody,
    lived: f32,
    countdown: f32,
}

impl ArmedGrenade {
    pub const GRENADE_COUNTDOWN_DURATION: f32 = 0.5;
    pub const EXPLOSION_WIDTH: f32 = 100.0;
    pub const EXPLOSION_HEIGHT: f32 = 100.0;

    pub fn new(pos: Vec2, facing: bool) -> Self {
        // TODO: In case we want to animate thrown grenades rotating etc.
        let grenade_sprite = AnimatedSprite::new(
            15,
            15,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                },
            ],
            false,
        );

        let speed = if facing {
            vec2(600., -200.)
        } else {
            vec2(-600., -200.)
        };

        let mut body = PhysicsBody {
            pos,
            facing,
            angle: 0.0,
            speed,
            collider: None,
            on_ground: false,
            last_frame_on_ground: false,
            have_gravity: true,
        };

        let mut resources = storage::get_mut::<Resources>();

        let grenade_mount_pos = if facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        body.collider = Some(resources.collision_world.add_actor(
            body.pos + grenade_mount_pos,
            15,
            15,
        ));

        ArmedGrenade {
            grenade_sprite,
            body,
            lived: 0.0,
            countdown: Self::GRENADE_COUNTDOWN_DURATION,
        }
    }
}

pub struct ArmedGrenades {
    grenades: Vec<ArmedGrenade>,
}

impl ArmedGrenades {
    pub fn new() -> Self {
        ArmedGrenades {
            grenades: Vec::with_capacity(200),
        }
    }

    pub fn spawn_grenade(&mut self, pos: Vec2, facing: bool) {
        self.grenades.push(ArmedGrenade::new(pos, facing));
    }
}

impl scene::Node for ArmedGrenades {
    fn fixed_update(mut node: RefMut<Self>) {
        for grenade in &mut node.grenades {
            grenade.body.update();
            grenade.lived += get_frame_time();
        }

        node.grenades.retain(|grenade| {
            if grenade.lived >= grenade.countdown {
                {
                    let mut resources = storage::get_mut::<Resources>();
                    resources.hit_fxses.spawn(grenade.body.pos);
                }
                let grenade_rect = Rect::new(
                    grenade.body.pos.x - (ArmedGrenade::EXPLOSION_WIDTH / 2.0),
                    grenade.body.pos.y - (ArmedGrenade::EXPLOSION_HEIGHT / 2.0),
                    ArmedGrenade::EXPLOSION_WIDTH,
                    ArmedGrenade::EXPLOSION_HEIGHT,
                );
                for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                    let intersect =
                        grenade_rect.intersect(Rect::new(
                            player.body.pos.x,
                            player.body.pos.y,
                            20.0,
                            64.0,
                        ));
                    if !intersect.is_none() {
                        let direction = grenade.body.pos.x > (player.body.pos.x + 10.);
                        scene::find_node_by_type::<crate::nodes::Camera>()
                            .unwrap()
                            .shake();
                        player.kill(direction);
                    }
                }
                return false;
            }
            return true;
        });
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();
        for grenade in &node.grenades {
            draw_texture_ex(
                resources.grenades,
                grenade.body.pos.x,
                grenade.body.pos.y,
                color::WHITE,
                DrawTextureParams {
                    source: Some(grenade.grenade_sprite.frame().source_rect),
                    dest_size: Some(grenade.grenade_sprite.frame().dest_size),
                    flip_x: false,
                    rotation: 0.0,
                    ..Default::default()
                },
            );
        }
    }
}
