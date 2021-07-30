use macroquad::{
    experimental::{
        scene::{
            Node,
            RefMut,
            HandleUntyped,
            Handle,
        },
        animation::{
            AnimatedSprite,
            Animation,
        },
        collections::storage,
        coroutines::{
            Coroutine,
            start_coroutine,
        },
    },
    audio::play_sound_once,
    color,
    prelude::*,
};

use crate::{
    Resources,
    nodes::{
        player::{
            Player,
            capabilities,
            PhysicsBody,
            Weapon,
        },
        sproinger::Sproingable,
    }
};

pub struct Crate {
    pub sprite: AnimatedSprite,
    pub body: PhysicsBody,

    pub thrown: bool,

    origin_pos: Vec2,
    deadly_dangerous: bool,
}

impl Crate {
    pub fn new(facing: bool, pos: Vec2) -> Self {
        let sprite = AnimatedSprite::new(
            32,
            32,
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

        let body= PhysicsBody {
            pos,
            facing,
            angle: 0.0,
            speed: vec2(0., 0.),
            collider: None,
            on_ground: false,
            last_frame_on_ground: false,
            have_gravity: true,
            bouncyness: 0.0,
        };

        Crate{
            sprite,
            body,
            thrown: false,
            origin_pos: pos,
            deadly_dangerous: false,
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

        let mount_pos = if self.body.facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        if self.body.collider.is_none() {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos + mount_pos,
                30,
                30,
            ));
        } else {
            resources
                .collision_world
                .set_actor_position(self.body.collider.unwrap(), self.body.pos + mount_pos);
        }

        self.origin_pos = self.body.pos + mount_pos / 2.;
    }

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Crate>();

            Crate::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            start_coroutine(async move {
                let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Crate>();

                Crate::throw(&mut *node, true);

                let player = &mut *scene::get_node(player);
                player.weapon = None;
                player.state_machine.set_state(Player::ST_NORMAL);
            })
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Crate>();

            node.thrown
        }

        fn pick_up(node: HandleUntyped) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Crate>();

            node.body.angle = 0.;
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

impl Node for Crate {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(30.0, 30.0),
            Self::gun_capabilities(),
        ));

        node.provides::<Sproingable>((
           node.handle().untyped(),
           node.handle().lens(|node| &mut node.body),
           vec2(30.0, 30.0),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        if node.thrown {
            node.body.update();
            node.body.update_throw();

            if (node.origin_pos - node.body.pos).length() > 70. {
                node.deadly_dangerous = true;
            }
            if node.body.speed.length() <= 200.0 {
                node.deadly_dangerous = false;
            }
            // if node.body.on_ground {
            //     node.deadly_dangerous = false;
            // }

            if node.deadly_dangerous {
                let players = scene::find_nodes_by_type::<crate::nodes::Player>();
                let hit_box = Rect::new(node.body.pos.x, node.body.pos.y, 30., 30.);

                for mut player in players {
                    let player_hitbox = player.get_hitbox();
                    let is_overlapping = hit_box.overlaps(&player_hitbox);
                    if is_overlapping {
                        if node.body.pos.y + 30.0 < player_hitbox.y + Player::HEAD_THRESHOLD {
                            let resources = storage::get_mut::<Resources>();
                            play_sound_once(resources.jump_sound);
                            player.kill(!node.body.facing);
                        }
                    }
                }
            }
        }
    }

    fn draw(mut node: RefMut<Self>) {
        node.sprite.update();

        let resources = storage::get_mut::<Resources>();

        let mount_pos = if node.thrown {
            if node.body.facing {
                vec2(-24., 0.)
            } else {
                vec2(5., 0.)
            }
        } else {
            if node.body.facing {
                vec2(24., 16.)
            } else {
                vec2(-24., 16.)
            }
        };

        draw_texture_ex(
            resources.crates,
            node.body.pos.x + mount_pos.x,
            node.body.pos.y + mount_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.sprite.frame().source_rect),
                dest_size: Some(node.sprite.frame().dest_size),
                flip_x: !node.body.facing,
                rotation: node.body.angle,
                ..Default::default()
            },
        );
    }
}
