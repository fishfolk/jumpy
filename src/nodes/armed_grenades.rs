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

use crate::Resources;

pub struct ArmedGrenade {
    grenade_sprite: AnimatedSprite,
    pos: Vec2,
    speed: Vec2,
    lived: f32,
    countdown: f32,
}

impl ArmedGrenade {
    pub const GRENADE_SPEED: f32 = 500.0;
    pub const GRENADE_COUNTDOWN_DURATION: f32 = 5.0;

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
                Animation {
                    name: "shoot".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                },
            ],
            false,
        );

        let dir = if facing {
            vec2(1.0, -0.5)
        } else {
            vec2(-1.0, -0.5)
        };

        ArmedGrenade {
            grenade_sprite,
            pos: pos + vec2(16.0, 30.0) + dir * 32.0,
            speed: dir * Self::GRENADE_SPEED,
            lived: 0.0,
            countdown: Self::GRENADE_COUNTDOWN_DURATION,
        }
    }
}

pub struct ArmedGrenades {
    grenades: Vec<ArmedGrenade>,
}

impl ArmedGrenades {
    pub const GRAVITY: f32 = 50.0;

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
        let mut resources = storage::get_mut::<Resources>();

        for grenade in &mut node.grenades {
            grenade.speed.y += Self::GRAVITY;
            grenade.pos += grenade.speed * get_frame_time();
            grenade.lived += get_frame_time();
        }

        node.grenades.retain(|grenade| {
            let mut killed = false;
            for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                let self_damaged =
                    Rect::new(player.body.pos.x, player.body.pos.y, 20., 64.).contains(grenade.pos);
                let direction = grenade.pos.x > (player.body.pos.x + 10.);

                if self_damaged {
                    killed = true;

                    scene::find_node_by_type::<crate::nodes::Camera>()
                        .unwrap()
                        .shake();

                    player.kill(direction);
                }
            }

            if resources.collision_world.solid_at(grenade.pos) || killed {
                resources.hit_fxses.spawn(grenade.pos);
                return false;
            }
            grenade.lived < grenade.countdown
        });
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();
        for grenade in &node.grenades {
            draw_texture_ex(
                resources.grenades,
                grenade.pos.x,
                grenade.pos.y,
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
