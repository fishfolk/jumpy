use macroquad::{
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, Coroutine},
        scene::{Handle, HandleUntyped, Node, RefMut},
    },
    prelude::*,
};

use crate::{
    capabilities,
    components::{FlyingGalleon, GunlikeAnimation, PhysicsBody, ThrowableItem},
    nodes::Player,
    Resources,
};

pub struct Galleon {
    sprite: GunlikeAnimation,

    body: PhysicsBody,
    throwable: ThrowableItem,
}

impl Galleon {
    pub const COLLIDER_WIDTH: f32 = 32.0;
    pub const COLLIDER_HEIGHT: f32 = 29.0;

    pub fn throw(&mut self, force: bool) {
        self.throwable.throw(&mut self.body, force);
    }

    pub fn shoot(node: Handle<Galleon>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let player = &mut *scene::get_node(player);
                scene::add_node(FlyingGalleon::new(
                    player.id
                ));
            }

            {
                let player = &mut *scene::get_node(player);
                player.state_machine.set_state(Player::ST_NORMAL);

                player.weapon = None;
                scene::get_node(node).delete();
            }
        };

        start_coroutine(coroutine)
    }

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let mut resources = storage::get_mut::<Resources>();

        let sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                32,
                29,
                &[
                    Animation {
                        name: "idle".to_string(),
                        row: 0,
                        frames: 1,
                        fps: 1,
                    },
                ],
                false,
            ),
            resources.items_textures["galleon/galleon"],
            Self::COLLIDER_WIDTH,
        );

        let body = PhysicsBody::new(
            &mut resources.collision_world,
            pos,
            0.0,
            vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT),
        );

        scene::add_node(Galleon {
            sprite,
            body,
            throwable: ThrowableItem::default(),
        })
        .untyped()
    }

    pub fn weapon_capabilities() -> capabilities::Weapon {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Galleon>();

            Galleon::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Galleon>()
                .handle();
            Galleon::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Galleon>();

            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Galleon>();

            node.body.angle = 0.;
            node.throwable.owner = Some(owner);
        }

        fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Galleon>();
            let mount_pos = if node.body.facing {
                vec2(15., 16.)
            } else {
                vec2(-22., 16.)
            };

            node.body.pos = parent_pos + mount_pos;
            node.body.facing = parent_facing;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Galleon>();
            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                Galleon::COLLIDER_WIDTH,
                Galleon::COLLIDER_HEIGHT,
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
                .to_typed::<Galleon>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Galleon>();

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
                .to_typed::<Galleon>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Galleon>();
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

impl Node for Galleon {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::weapon_capabilities());
        node.provides(Self::physics_capabilities());
    }

    fn fixed_update(mut node: RefMut<Self>) {
        let node = &mut *node;

        node.sprite.update();
        node.throwable.update(&mut node.body, true);
    }

    fn draw(node: RefMut<Self>) {
        node.sprite
            .draw(node.body.pos, node.body.facing, node.body.angle);
    }
}
