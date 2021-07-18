use macroquad::{
    audio::play_sound_once,
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, Handle, HandleUntyped, RefMut},
    },
    prelude::*,
};

use crate::{
    nodes::player::{capabilities, PhysicsBody, Weapon},
    nodes::Player,
    Resources,
};

pub struct Muscet {
    pub muscet_sprite: AnimatedSprite,
    pub muscet_fx_sprite: AnimatedSprite,
    pub muscet_fx: bool,

    pub thrown: bool,

    pub bullets: i32,
    pub body: PhysicsBody,

    origin_pos: Vec2,
    pub deadly_dangerous: bool,
}

impl scene::Node for Muscet {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            Self::gun_capabilities(),
        ));
    }

    fn draw(mut node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        let muscet_mount_pos = if node.thrown == false {
            if node.body.facing {
                vec2(0., 16.)
            } else {
                vec2(-60., 16.)
            }
        } else {
            if node.body.facing {
                vec2(-25., 0.)
            } else {
                vec2(5., 0.)
            }
        };

        draw_texture_ex(
            resources.gun,
            node.body.pos.x + muscet_mount_pos.x,
            node.body.pos.y + muscet_mount_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.muscet_sprite.frame().source_rect),
                dest_size: Some(node.muscet_sprite.frame().dest_size),
                flip_x: !node.body.facing,
                rotation: node.body.angle,
                ..Default::default()
            },
        );

        if node.muscet_fx {
            node.muscet_fx_sprite.update();
            draw_texture_ex(
                resources.gun,
                node.body.pos.x + muscet_mount_pos.x,
                node.body.pos.y + muscet_mount_pos.y,
                color::WHITE,
                DrawTextureParams {
                    source: Some(node.muscet_fx_sprite.frame().source_rect),
                    dest_size: Some(node.muscet_fx_sprite.frame().dest_size),
                    flip_x: !node.body.facing,
                    ..Default::default()
                },
            );
        }

        // if let Some(collider) = node.collider {
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

        if node.thrown == false {
            node.draw_hud();
        }
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.muscet_sprite.update();

        if node.thrown {
            node.body.update();
            node.body.update_throw();

            if (node.origin_pos - node.body.pos).length() > 70. {
                node.deadly_dangerous = true;
            }
            if node.body.speed.length() <= 200.0 {
                node.deadly_dangerous = false;
            }
            if node.body.on_ground {
                node.deadly_dangerous = false;
            }

            if node.deadly_dangerous {
                let others = scene::find_nodes_by_type::<crate::nodes::Player>();
                let sword_hit_box = Rect::new(node.body.pos.x - 10., node.body.pos.y, 60., 30.);

                for mut other in others {
                    if Rect::new(other.body.pos.x, other.body.pos.y, 20., 64.)
                        .overlaps(&sword_hit_box)
                    {
                        other.kill(!node.body.facing);
                    }
                }
            }
        }
    }
}

impl Muscet {
    pub const GUN_THROWBACK: f32 = 700.0;

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
            body: PhysicsBody {
                pos,
                facing,
                angle: 0.0,
                speed: vec2(0., 0.),
                collider: None,
                on_ground: false,
                last_frame_on_ground: false,
                have_gravity: true,
            },
            thrown: false,
            bullets: 3,
            origin_pos: pos,
            deadly_dangerous: false,
        }
    }

    fn draw_hud(&self) {
        let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
        let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);
        for i in 0..3 {
            let x = self.body.pos.x + 15.0 * i as f32;

            if i >= self.bullets {
                draw_circle_lines(x, self.body.pos.y - 12.0, 4.0, 2., empty_color);
            } else {
                draw_circle(x, self.body.pos.y - 12.0, 4.0, full_color);
            };
        }
    }

    pub fn throw(&mut self, force: bool) {
        self.thrown = true;

        if force {
            self.body.speed = if self.body.facing {
                vec2(600., -200.)
            } else {
                vec2(-600., -200.)
            };
        } else {
            self.body.angle = 3.5;
        }

        let mut resources = storage::get_mut::<Resources>();

        let sword_mount_pos = if self.body.facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        if self.body.collider.is_none() {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos + sword_mount_pos,
                40,
                30,
            ));
        } else {
            resources
                .collision_world
                .set_actor_position(self.body.collider.unwrap(), self.body.pos + sword_mount_pos);
        }
        self.origin_pos = self.body.pos + sword_mount_pos / 2.;
    }

    pub fn shoot(node: Handle<Muscet>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let node = scene::get_node(node);
                if node.bullets <= 0 {
                    let player = &mut *scene::get_node(player);
                    player.state_machine.set_state(Player::ST_NORMAL);

                    return;
                }
            }

            // {
            //     scene::find_node_by_type::<crate::nodes::Camera>()
            //         .unwrap()
            //         .shake();
            // }
            {
                let resources = storage::get_mut::<Resources>();
                play_sound_once(resources.shoot_sound);

                let mut node = &mut *scene::get_node(node);
                let player = &mut *scene::get_node(player);

                node.muscet_fx = true;

                let mut bullets = scene::find_node_by_type::<crate::nodes::Bullets>().unwrap();
                bullets.spawn_bullet(node.body.pos, node.body.facing);
                player.body.speed.x = -Self::GUN_THROWBACK * player.body.facing_dir();
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

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Muscet>();

            Muscet::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Muscet>()
                .handle();

            Muscet::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Muscet>();

            node.thrown
        }

        fn pick_up(node: HandleUntyped) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Muscet>();

            node.body.angle = 0.;
            node.bullets = 3;
            node.thrown = false;
        }
        capabilities::Gun {
            throw,
            shoot,
            is_thrown,
            pick_up,
        }
    }
}
