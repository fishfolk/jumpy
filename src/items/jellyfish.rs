use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, Coroutine},
        scene::{self, Handle, HandleUntyped, RefMut},
    },
    prelude::*,
};

use crate::{capabilities, components::{PhysicsBody, ThrowableItem}, nodes::flappy_jellyfish::FlappyJellyfish, nodes::Player, Resources, GameWorld};

/// Statuses, in order
#[derive(Copy, Clone, Debug)]
pub enum MountStatus {
    // This is the normal sequence of statuses. Death will reset the state to Dropped
    Dropped,
    Mounted,
    Driving,
    Dismounted,
}

pub struct Jellyfish {
    pub sprite: AnimatedSprite,

    pub body: PhysicsBody,
    pub throwable: ThrowableItem,
    pub used: bool,
    pub mount_status: MountStatus,
}

impl scene::Node for Jellyfish {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::weapon_capabilities());
        node.provides(Self::physics_capabilities());
    }

    fn draw(mut node: RefMut<Self>) {
        node.sprite.update();

        let resources = storage::get::<Resources>();

        draw_texture_ex(
            resources.items_textures["jellyfish/jellyfish"],
            node.body.pos.x,
            node.body.pos.y,
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

    fn fixed_update(mut node: RefMut<Self>) {
        let node = &mut *node;

        node.sprite.update();
        node.throwable.update(&mut node.body, true);
    }
}

impl Jellyfish {
    const COLLIDER_WIDTH: f32 = 30.;
    const COLLIDER_HEIGHT: f32 = 39.;

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let sprite = AnimatedSprite::new(
            30,
            39,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                },
                Animation {
                    name: "put-on".to_string(),
                    row: 1,
                    frames: 1,
                    fps: 1,
                },
            ],
            false,
        );

        let mut world = storage::get_mut::<GameWorld>();

        scene::add_node(Jellyfish {
            sprite,
            body: PhysicsBody::new(
                &mut world.collision_world,
                pos,
                0.0,
                vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT),
            ),
            throwable: ThrowableItem::default(),
            used: false,
            mount_status: MountStatus::Mounted,
        })
        .untyped()
    }

    pub fn throw(&mut self, force: bool) {
        self.mount_status = MountStatus::Dropped;
        self.sprite.set_animation(0);
        self.throwable.throw(&mut self.body, force);
    }

    pub fn shoot(node_h: Handle<Jellyfish>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let mut node = scene::get_node(node_h);
                let player = &mut *scene::get_node(player);

                match node.mount_status {
                    MountStatus::Mounted => {
                        let was_spawned = FlappyJellyfish::spawn(&mut *node, player);

                        if !was_spawned {
                            player.state_machine.set_state(Player::ST_NORMAL);
                        }
                    }
                    MountStatus::Dismounted => {
                        Jellyfish::throw(&mut *node, true);
                        player.weapon = None;
                        player.state_machine.set_state(Player::ST_NORMAL);
                    }

                    _ => panic!("Unexpected jellyfish mount status: {:?}", node.mount_status),
                }
            }
        };

        start_coroutine(coroutine)
    }

    pub fn weapon_capabilities() -> capabilities::Weapon {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Jellyfish>();

            Jellyfish::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Jellyfish>()
                .handle();

            Jellyfish::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Jellyfish>();

            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Jellyfish>();

            node.body.angle = 0.;
            node.throwable.owner = Some(owner);
            node.sprite.set_animation(1);
            node.mount_status = MountStatus::Mounted;
        }

        fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Jellyfish>();

            let mount_pos: Vec2;
            mount_pos = if node.body.facing {
                vec2(-14., -10.)
            } else {
                vec2(10., -10.)
            };

            node.body.pos = parent_pos + mount_pos;
            node.body.facing = parent_facing;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Jellyfish>();
            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                Jellyfish::COLLIDER_WIDTH,
                Jellyfish::COLLIDER_HEIGHT,
            )
        }

        capabilities::Weapon {
            collider,
            mount,
            is_thrown,
            pick_up,
            throw,
            shoot,
        }
    }

    fn physics_capabilities() -> capabilities::PhysicsObject {
        fn active(handle: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Jellyfish>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Jellyfish>();

            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                node.body.size.x,
                node.body.size.y,
            )
        }
        fn set_speed_x(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Jellyfish>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Jellyfish>();
            node.body.speed.y = speed;
        }

        capabilities::PhysicsObject {
            active,
            collider,
            set_speed_x,
            set_speed_y,
        }
    }
}
