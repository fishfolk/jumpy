use macroquad::{
    experimental::{
        animation::{
            AnimatedSprite,
            Animation,
        },
        scene::{
            RefMut,
            Node,
            Handle,
            HandleUntyped,
        },
        coroutines::{
            Coroutine,
            wait_seconds,
            start_coroutine,
        },
        collections::storage,
    },
    color,
    audio::play_sound_once,
    prelude::*,
};

use crate::{
    Resources,
    nodes::{
        sproinger::Sproingable,
        player::{
            Player,
            Weapon,
            PhysicsBody,
            capabilities,
        }
    }
};

pub struct ArmedKickBomb {
    pub sprite: AnimatedSprite,
    pub body: PhysicsBody,
    pub lived: f32,
}

impl ArmedKickBomb {
    pub const COUNTDOWN_DURATION: f32 = 0.5;
    pub const EXPLOSION_WIDTH: f32 = 100.0;
    pub const EXPLOSION_HEIGHT: f32 = 100.0;

    pub fn new(pos: Vec2, facing: bool) -> Self {
        let sprite = AnimatedSprite::new(
            32,
            36,
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

        let body = PhysicsBody {
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

        ArmedKickBomb {
            sprite,
            body,
            lived: 0.0,
        }
    }

    pub fn spawn(pos: Vec2, facing: bool) {
        let bomb = ArmedKickBomb::new(pos, facing);
        scene::add_node(bomb);
    }
}

impl Node for ArmedKickBomb {

    fn ready(mut node: RefMut<Self>) {
        node.provides::<Sproingable>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(16.0, 32.0),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.body.update();
        node.lived += get_frame_time();

        if node.lived >= ArmedKickBomb::COUNTDOWN_DURATION {
            {
                let mut resources = storage::get_mut::<Resources>();
                resources.hit_fxses.spawn(node.body.pos);
            }
            let grenade_rect = Rect::new(
                node.body.pos.x - (ArmedKickBomb::EXPLOSION_WIDTH / 2.0),
                node.body.pos.y - (ArmedKickBomb::EXPLOSION_HEIGHT / 2.0),
                ArmedKickBomb::EXPLOSION_WIDTH,
                ArmedKickBomb::EXPLOSION_HEIGHT,
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
                    let direction = node.body.pos.x > (player.body.pos.x + 10.);
                    scene::find_node_by_type::<crate::nodes::Camera>()
                        .unwrap()
                        .shake();
                    player.kill(direction);
                }
            }
            node.delete();
        }
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();
        draw_texture_ex(
            resources.kick_bomb,
            node.body.pos.x,
            node.body.pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.sprite.frame().source_rect),
                dest_size: Some(node.sprite.frame().dest_size),
                flip_x: false,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}

pub struct KickBomb {
    pub sprite: AnimatedSprite,
    pub body: PhysicsBody,

    pub thrown: bool,

    origin_pos: Vec2,
    pub deadly_dangerous: bool,
}

impl KickBomb {
    pub fn new(facing: bool, pos: Vec2) -> Self {
        let sprite = AnimatedSprite::new(
            32,
            36,
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

        let body = PhysicsBody {
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

        KickBomb {
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
        self.origin_pos = self.body.pos + mount_pos / 2.;
    }

    pub fn shoot(node: Handle<KickBomb>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            // This should be cleaned up. Animation is doing nothing but acting as a
            // cooldown between throws now

            {
                let node = scene::get_node(node);

                ArmedKickBomb::spawn(node.body.pos, node.body.facing);
            }

            wait_seconds(0.08).await;

            {
                let player = &mut *scene::get_node(player);
                player.state_machine.set_state(Player::ST_NORMAL);
            }
        };

        start_coroutine(coroutine)
    }

    pub fn gun_capabilities() -> capabilities::Gun {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<KickBomb>();

            KickBomb::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<KickBomb>()
                .handle();

            KickBomb::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<KickBomb>();

            node.thrown
        }

        fn pick_up(node: HandleUntyped) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<KickBomb>();

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

impl Node for KickBomb {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<Weapon>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            Self::gun_capabilities(),
        ));
        node.provides::<Sproingable>((
            node.handle().untyped(),
            node.handle().lens(|node| &mut node.body),
            vec2(32.0, 32.0),
        ));
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.sprite.update();

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

    fn draw(mut node: RefMut<KickBomb>) {
        let resources = storage::get_mut::<Resources>();

        let mount_pos = if node.thrown == false {
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
            resources.kick_bomb,
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
