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
    nodes::player::{capabilities, PhysicsBody, PhysicsObject, Weapon},
    nodes::sproinger::Sproingable,
    nodes::Player,
    Resources,
};

pub struct Sword {
    pub sword_sprite: AnimatedSprite,
    pub thrown: bool,

    pub body: PhysicsBody,

    origin_pos: Vec2,
    pub deadly_dangerous: bool,

    // hack, just for swordthrow loc
    spawn_pos: (Vec2, bool),
}

impl scene::Node for Sword {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<PhysicsObject>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
        ));

        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            Self::gun_capabilities(),
        ));

        node.provides::<Sproingable>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(48.0, 32.0),
        ));
    }

    fn draw(sword: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        //sword.dead == false && matches!(sword.weapon, Some(Weapon::Sword)) {
        // for attack animation - old, pre-rotated sprite
        if sword.sword_sprite.current_animation() == 1 {
            let sword_mount_pos = if sword.body.facing {
                vec2(10., -35.)
            } else {
                vec2(-50., -35.)
            };
            draw_texture_ex(
                resources.sword,
                sword.body.pos.x + sword_mount_pos.x,
                sword.body.pos.y + sword_mount_pos.y,
                color::WHITE,
                DrawTextureParams {
                    source: Some(sword.sword_sprite.frame().source_rect),
                    dest_size: Some(sword.sword_sprite.frame().dest_size),
                    flip_x: !sword.body.facing,
                    ..Default::default()
                },
            );
        } else {
            // just casually holding a sword

            let sword_mount_pos = if sword.thrown == false {
                if sword.body.facing {
                    vec2(5., 10.)
                } else {
                    vec2(-45., 10.)
                }
            } else {
                if sword.body.facing {
                    vec2(-25., 0.)
                } else {
                    vec2(5., 0.)
                }
            };

            let rotation = if sword.body.facing {
                -sword.body.angle
            } else {
                sword.body.angle
            };

            draw_texture_ex(
                resources.fish_sword,
                sword.body.pos.x + sword_mount_pos.x,
                sword.body.pos.y + sword_mount_pos.y,
                color::WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(65., 17.)),
                    flip_x: !sword.body.facing,
                    rotation: rotation, //get_time() as _,
                    ..Default::default()
                },
            );
        }

        if let Some(_collider) = sword.body.collider {
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

    fn fixed_update(mut node: RefMut<Self>) {
        node.sword_sprite.update();

        let map_bottom = {
            let resources = storage::get::<Resources>();

            resources.tiled_map.raw_tiled_map.tileheight * resources.tiled_map.raw_tiled_map.height
        } as f32;
        // respawn sword
        // should not be here, just a hack for swordthrow loc
        if node.body.pos.y > map_bottom {
            node.body.pos = node.spawn_pos.0;
            node.body.facing = node.spawn_pos.1;
            node.body.speed = vec2(0., 0.);
            node.deadly_dangerous = false;
            node.throw(false);
            return;
        }

        if node.thrown {
            node.body.update();
            node.body.update_throw();

            if (node.origin_pos - node.body.pos).length() > 70. {
                node.deadly_dangerous = true;
            }
            if node.body.speed.length() <= 200.0 {
                node.deadly_dangerous = false;
            }
            if node.body.on_ground && node.body.speed.length() <= 400.0 {
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
            body: PhysicsBody {
                pos,
                facing,
                speed: vec2(0., 0.),
                angle: std::f32::consts::PI / 4. + 0.3,
                collider: None,
                last_frame_on_ground: false,
                on_ground: false,
                have_gravity: true,
                bouncyness: 0.0,
            },
            thrown: false,
            origin_pos: pos,
            deadly_dangerous: false,
            spawn_pos: (pos, facing),
        }
    }

    pub fn throw(&mut self, force: bool) {
        self.thrown = true;

        if force {
            self.body.speed = if self.body.facing {
                vec2(800., -50.)
            } else {
                vec2(-800., -50.)
            };
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

    pub fn shoot(node: Handle<Sword>, player: Handle<Player>) -> Coroutine {
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
                let sword_hit_box = if player.body.facing {
                    Rect::new(player.body.pos.x + 35., player.body.pos.y - 5., 40., 60.)
                } else {
                    Rect::new(player.body.pos.x - 50., player.body.pos.y - 5., 40., 60.)
                };

                for mut other in others {
                    if Rect::new(other.body.pos.x, other.body.pos.y, 20., 64.)
                        .overlaps(&sword_hit_box)
                    {
                        scene::find_node_by_type::<crate::nodes::Camera>()
                            .unwrap()
                            .shake();
                        other.kill(!player.body.facing);
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

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Sword>();

            Sword::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Sword>()
                .handle();

            Sword::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Sword>();

            node.thrown
        }

        fn pick_up(node: HandleUntyped) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Sword>();

            node.body.angle = std::f32::consts::PI / 4. + 0.3;
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
