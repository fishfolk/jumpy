use macroquad::{
    audio::play_sound_once,
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, Handle, RefMut},
    },
    prelude::*,
};
use macroquad_platformer::Actor;

use crate::{nodes::Player, Resources};

pub struct Muscet {
    pub muscet_sprite: AnimatedSprite,
    pub muscet_fx_sprite: AnimatedSprite,
    pub muscet_fx: bool,

    pub facing: bool,
    pub pos: Vec2,
    pub thrown: bool,
    pub angle: f32,

    pub bullets: i32,

    speed: Vec2,
    collider: Option<Actor>,
    origin_pos: Vec2,
    deadly_dangerous: bool,
}

impl scene::Node for Muscet {
    fn draw(mut muscet: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        let muscet_mount_pos = if muscet.thrown == false {
            if muscet.facing {
                vec2(0., 16.)
            } else {
                vec2(-60., 16.)
            }
        } else {
            if muscet.facing {
                vec2(-25., 0.)
            } else {
                vec2(5., 0.)
            }
        };

        draw_texture_ex(
            resources.gun,
            muscet.pos.x + muscet_mount_pos.x,
            muscet.pos.y + muscet_mount_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(muscet.muscet_sprite.frame().source_rect),
                dest_size: Some(muscet.muscet_sprite.frame().dest_size),
                flip_x: !muscet.facing,
                rotation: muscet.angle,
                ..Default::default()
            },
        );

        if muscet.muscet_fx {
            muscet.muscet_fx_sprite.update();
            draw_texture_ex(
                resources.gun,
                muscet.pos.x + muscet_mount_pos.x,
                muscet.pos.y + muscet_mount_pos.y,
                color::WHITE,
                DrawTextureParams {
                    source: Some(muscet.muscet_fx_sprite.frame().source_rect),
                    dest_size: Some(muscet.muscet_fx_sprite.frame().dest_size),
                    flip_x: !muscet.facing,
                    ..Default::default()
                },
            );
        }

        // if let Some(collider) = muscet.collider {
        //     let pos = resources.collision_world.actor_pos(collider);

        //     let sword_hit_box = Rect::new(pos.x, pos.y, 40., 30.);
        //     draw_rectangle(
        //         sword_hit_box.x,
        //         sword_hit_box.y,
        //         sword_hit_box.w,
        //         sword_hit_box.h,
        //         RED,
        //     );
        // }
    }

    fn update(mut node: RefMut<Self>) {
        node.muscet_sprite.update();

        if node.thrown {
            let collider = node.collider.unwrap();
            let mut resources = storage::get_mut::<Resources>();
            node.pos = resources.collision_world.actor_pos(collider);

            if (node.origin_pos - node.pos).length() > 70. {
                node.deadly_dangerous = true;
            }
            let on_ground = resources
                .collision_world
                .collide_check(collider, node.pos + vec2(0., 5.));

            if on_ground == false {
                node.angle += node.speed.x.abs() * 0.00015 + node.speed.y.abs() * 0.00005;

                node.speed.y += crate::consts::GRAVITY * get_frame_time();
            } else {
                node.deadly_dangerous = false;

                node.angle %= std::f32::consts::PI * 2.;
                let goal = if node.angle <= std::f32::consts::PI {
                    std::f32::consts::PI
                } else {
                    std::f32::consts::PI * 2.
                };

                let rest = goal - node.angle;
                if rest.abs() >= 0.1 {
                    node.angle += (rest * 0.1).max(0.1);
                }
            }

            node.speed.x *= 0.98;
            if node.speed.x.abs() <= 1. {
                node.speed.x = 0.0;
            }
            resources
                .collision_world
                .move_h(collider, node.speed.x * get_frame_time());
            if !resources
                .collision_world
                .move_v(collider, node.speed.y * get_frame_time())
            {
                node.speed.y = 0.0;
            }

            if node.deadly_dangerous {
                let others = scene::find_nodes_by_type::<crate::nodes::Player>();
                let sword_hit_box = Rect::new(node.pos.x - 10., node.pos.y, 60., 30.);

                for mut other in others {
                    if Rect::new(other.pos().x, other.pos().y, 20., 64.).overlaps(&sword_hit_box) {
                        other.kill(!node.facing);
                    }
                }
            }
        }
    }
}

impl Muscet {
    pub fn new(facing: bool, pos: Vec2) -> Muscet {
        let muscet_sprite = AnimatedSprite::new(
            92,
            32,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                },
                Animation {
                    name: "shoot".to_string(),
                    row: 1,
                    frames: 3,
                    fps: 15,
                },
            ],
            false,
        );
        let muscet_fx_sprite = AnimatedSprite::new(
            92,
            32,
            &[Animation {
                name: "shoot".to_string(),
                row: 2,
                frames: 3,
                fps: 15,
            }],
            false,
        );

        Muscet {
            muscet_sprite,
            muscet_fx_sprite,
            muscet_fx: false,
            pos,
            facing,
            thrown: false,
            bullets: 3,
            angle: 0.0,
            collider: None,
            origin_pos: pos,
            speed: vec2(0., 0.),
            deadly_dangerous: false,
        }
    }

    pub fn throw(&mut self, force: bool) {
        self.thrown = true;

        if force {
            self.speed = if self.facing {
                vec2(1600., -50.)
            } else {
                vec2(-1600., -50.)
            };
        } else {
            self.angle = 3.5;
        }

        let mut resources = storage::get_mut::<Resources>();

        let sword_mount_pos = if self.facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        self.collider = Some(resources.collision_world.add_actor(
            self.pos + sword_mount_pos,
            40,
            30,
        ));
        self.origin_pos = self.pos + sword_mount_pos / 2.;
    }

    pub fn shot(node: Handle<Muscet>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let resources = storage::get_mut::<Resources>();
                play_sound_once(resources.shoot_sound);

                let mut node = &mut *scene::get_node(node);
                let player = &mut *scene::get_node(player);

                node.muscet_fx = true;

                let mut bullets = scene::find_node_by_type::<crate::nodes::Bullets>().unwrap();
                bullets.spawn_bullet(node.pos, node.facing);
                player.fish.speed.x = -crate::consts::GUN_THROWBACK * player.fish.facing_dir();
            }
            {
                let node = &mut *scene::get_node(node);
                node.muscet_sprite.set_animation(1);
            }
            for i in 0u32..3 {
                {
                    let node = &mut *scene::get_node(node);
                    node.muscet_sprite.set_frame(i);
                    node.muscet_fx_sprite.set_frame(i);
                }

                wait_seconds(0.08).await;
            }
            {
                let mut node = scene::get_node(node);
                node.muscet_sprite.set_animation(0);
            }

            {
                let mut node = scene::get_node(node);
                node.muscet_fx = false;
                node.bullets -= 1;
            }

            {
                let player = &mut *scene::get_node(player);
                // node.weapon_animation.play(0, 0..5).await;
                // node.weapon_animation.play(0, 5..).await;
                player.state_machine.set_state(Player::ST_NORMAL);
            }
        };

        start_coroutine(coroutine)
    }
}
