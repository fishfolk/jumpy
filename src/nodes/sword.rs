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

pub struct Sword {
    pub sword_sprite: AnimatedSprite,
    pub facing: bool,
    pub pos: Vec2,
    pub thrown: bool,
    pub angle: f32,

    speed: Vec2,
    collider: Option<Actor>,
    origin_pos: Vec2,
    deadly_dangerous: bool,
}

impl scene::Node for Sword {
    fn draw(sword: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        //sword.dead == false && matches!(sword.weapon, Some(Weapon::Sword)) {
        // for attack animation - old, pre-rotated sprite
        if sword.sword_sprite.current_animation() == 1 {
            let sword_mount_pos = if sword.facing {
                vec2(10., -35.)
            } else {
                vec2(-50., -35.)
            };
            draw_texture_ex(
                resources.sword,
                sword.pos.x + sword_mount_pos.x,
                sword.pos.y + sword_mount_pos.y,
                color::WHITE,
                DrawTextureParams {
                    source: Some(sword.sword_sprite.frame().source_rect),
                    dest_size: Some(sword.sword_sprite.frame().dest_size),
                    flip_x: !sword.facing,
                    ..Default::default()
                },
            );
        } else {
            // just casually holding a sword

            let sword_mount_pos = if sword.thrown == false {
                if sword.facing {
                    vec2(5., 10.)
                } else {
                    vec2(-45., 10.)
                }
            } else {
                if sword.facing {
                    vec2(-25., 0.)
                } else {
                    vec2(5., 0.)
                }
            };

            let rotation = if sword.facing {
                -sword.angle
            } else {
                sword.angle
            };
            draw_texture_ex(
                resources.fish_sword,
                sword.pos.x + sword_mount_pos.x,
                sword.pos.y + sword_mount_pos.y,
                color::WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(65., 17.)),
                    flip_x: !sword.facing,
                    rotation: rotation, //get_time() as _,
                    ..Default::default()
                },
            );
        }

        if let Some(_collider) = sword.collider {
            //let pos = resources.collision_world.actor_pos(collider);

            // let sword_hit_box = Rect::new(pos.x, pos.y, 40., 30.);
            // draw_rectangle(
            //     sword_hit_box.x,
            //     sword_hit_box.y,
            //     sword_hit_box.w,
            //     sword_hit_box.h,
            //     RED,
            // );
        }
    }

    fn update(mut node: RefMut<Self>) {
        node.sword_sprite.update();
        if node.thrown {
            let collider = node.collider.unwrap();
            let mut resources = storage::get_mut::<Resources>();
            node.pos = resources.collision_world.actor_pos(collider);

            if (node.origin_pos - node.pos).length() > 70. {
                node.deadly_dangerous = true;
            }
            if node.speed.length() <= 200.0 {
                node.deadly_dangerous = false;
            }
            let on_ground = resources
                .collision_world
                .collide_check(collider, node.pos + vec2(0., 5.));

            if on_ground == false {
                node.angle += node.speed.x.abs() * 0.00015 + node.speed.y.abs() * 0.0001;

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

impl Sword {
    pub fn new(facing: bool, pos: Vec2) -> Sword {
        let sword_sprite = AnimatedSprite::new(
            65,
            93,
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
                    frames: 4,
                    fps: 15,
                },
            ],
            false,
        );

        Sword {
            sword_sprite,
            pos,
            facing,
            thrown: false,
            angle: std::f32::consts::PI / 4. + 0.3,
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
                vec2(800., -50.)
            } else {
                vec2(-800., -50.)
            };
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

    pub fn shot(node: Handle<Sword>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let resources = storage::get_mut::<Resources>();
                play_sound_once(resources.sword_sound);

                let sword = &mut *scene::get_node(node);
                sword.sword_sprite.set_animation(1);
            }

            {
                let player = &mut *scene::get_node(player);
                let others = scene::find_nodes_by_type::<crate::nodes::Player>();
                let sword_hit_box = if player.fish.facing {
                    Rect::new(player.pos().x + 35., player.pos().y - 5., 40., 60.)
                } else {
                    Rect::new(player.pos().x - 50., player.pos().y - 5., 40., 60.)
                };

                for mut other in others {
                    if Rect::new(other.pos().x, other.pos().y, 20., 64.).overlaps(&sword_hit_box) {
                        other.kill(!player.fish.facing);
                    }
                }
            }

            for i in 0u32..3 {
                {
                    let sword = &mut *scene::get_node(node);
                    sword.sword_sprite.set_frame(i);
                }

                wait_seconds(0.08).await;
            }

            {
                let mut sword = scene::get_node(node);
                sword.sword_sprite.set_animation(0);
            }

            let player = &mut *scene::get_node(player);
            player.state_machine.set_state(Player::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }
}
