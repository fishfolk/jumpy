use macroquad::{
    //audio::play_sound_once,
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
    nodes::{
        player::{capabilities, PhysicsBody, Weapon},
        sproinger::Sproingable,
        ArmedKickBomb, Player,
    },
    Resources,
};

pub struct KickBombs {
    sprite: AnimatedSprite,

    pub thrown: bool,

    pub body: PhysicsBody,
}

impl KickBombs {
    pub const COLLIDER_WIDTH: f32 = 30.0;
    pub const COLLIDER_HEIGHT: f32 = 30.0;
    pub const FIRE_INTERVAL: f32 = 0.25;

    pub fn new(facing: bool, pos: Vec2) -> Self {
        let sprite = AnimatedSprite::new(
            32,
            36,
            &[Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            false,
        );

        KickBombs {
            sprite,
            body: PhysicsBody {
                pos,
                facing,
                angle: 0.0,
                speed: vec2(0., 0.),
                collider: None,
                on_ground: false,
                last_frame_on_ground: false,
                have_gravity: true,
                bouncyness: 0.0,
            },
            thrown: false,
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

        let mount_pos = if self.body.facing {
            vec2(30., 10.)
        } else {
            vec2(-50., 10.)
        };

        if self.body.collider.is_none() {
            self.body.collider = Some(resources.collision_world.add_actor(
                self.body.pos + mount_pos,
                40,
                30,
            ));
        } else {
            resources
                .collision_world
                .set_actor_position(self.body.collider.unwrap(), self.body.pos + mount_pos);
        }
    }

    pub fn shoot(node: Handle<KickBombs>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let mut node = scene::get_node(node);
                let mut player = &mut *scene::get_node(player);
                player.weapon = None;

                ArmedKickBomb::spawn(node.body.pos, node.body.facing);

                node.delete();
            }

            wait_seconds(KickBombs::FIRE_INTERVAL).await;

            {
                let player = &mut *scene::get_node(player);
                player.state_machine.set_state(Player::ST_NORMAL);
            }
        };

        start_coroutine(coroutine)
    }

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<KickBombs>();

            KickBombs::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<KickBombs>()
                .handle();

            KickBombs::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<KickBombs>();

            node.thrown
        }

        fn pick_up(node: HandleUntyped) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<KickBombs>();

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

impl scene::Node for KickBombs {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT),
            Self::gun_capabilities(),
        ));

        node.provides::<Sproingable>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.sprite.update();

        if node.thrown {
            node.body.update();
            node.body.update_throw();

            if !node.body.on_ground {
                let hitbox = Rect::new(node.body.pos.x, node.body.pos.y, KickBombs::COLLIDER_WIDTH, KickBombs::COLLIDER_HEIGHT);
                for mut player in scene::find_nodes_by_type::<Player>() {
                    if hitbox.overlaps(&player.get_hitbox()) {
                        if let Some((weapon, _, _, gun)) = player.weapon.as_mut() {
                            (gun.throw)(*weapon, false);
                            player.weapon = None;
                        }
                    }
                }
            }
        }
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        let mount_pos = if node.thrown == false {
            if node.body.facing {
                vec2(0., 16.)
            } else {
                vec2(-5., 16.)
            }
        } else {
            if node.body.facing {
                vec2(-25., 0.)
            } else {
                vec2(5., 0.)
            }
        };

        draw_texture_ex(
            resources.kick_bombs,
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
