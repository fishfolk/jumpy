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
    components::{GunlikeAnimation, PhysicsBody, ThrowableItem},
    nodes::Player,
    Resources,
};

pub struct SharkRain {
    sprite: GunlikeAnimation,

    body: PhysicsBody,
    throwable: ThrowableItem,
}

impl SharkRain {
    pub const SPRITE_WIDTH: u32 = 32;
    pub const SPRITE_HEIGHT: u32 = 34;
    pub const RIGHT_OFFSET: [f32; 2] = [15., 16.];
    pub const LEFT_OFFSET: [f32; 2] = [-22., 16.];

    pub fn throw(&mut self, force: bool) {
        self.throwable.throw(&mut self.body, force);
    }

    pub fn shoot(node: Handle<SharkRain>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            let player = &mut *scene::get_node(player);
            player.state_machine.set_state(Player::ST_NORMAL);

            player.weapon = None;
            scene::get_node(node).delete();
        };

        start_coroutine(coroutine)
    }

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let mut resources = storage::get_mut::<Resources>();

        let sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                Self::SPRITE_WIDTH,
                Self::SPRITE_HEIGHT,
                &[Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                }],
                false,
            ),
            resources.items_textures["shark_rain/shark_rain"],
            Self::SPRITE_WIDTH as f32,
        );

        let body = PhysicsBody::new(
            &mut resources.collision_world,
            pos,
            0.0,
            vec2(Self::SPRITE_WIDTH as f32, Self::SPRITE_HEIGHT as f32),
        );

        scene::add_node(SharkRain {
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
                .to_typed::<SharkRain>();

            SharkRain::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<SharkRain>()
                .handle();
            SharkRain::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<SharkRain>();

            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<SharkRain>();

            node.body.angle = 0.;
            node.throwable.owner = Some(owner);
        }

        fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<SharkRain>();
            let mount_pos = if node.body.facing {
                vec2(SharkRain::RIGHT_OFFSET[0], SharkRain::RIGHT_OFFSET[1])
            } else {
                vec2(SharkRain::LEFT_OFFSET[0], SharkRain::LEFT_OFFSET[1])
            };

            node.body.pos = parent_pos + mount_pos;
            node.body.facing = parent_facing;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<SharkRain>();
            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                SharkRain::SPRITE_WIDTH as f32,
                SharkRain::SPRITE_HEIGHT as f32,
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
                .to_typed::<SharkRain>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<SharkRain>();

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
                .to_typed::<SharkRain>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<SharkRain>();
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

impl Node for SharkRain {
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
