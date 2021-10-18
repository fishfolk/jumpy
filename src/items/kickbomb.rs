use macroquad::{
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, Coroutine},
        scene::{self, Handle, HandleUntyped, RefMut},
    },
    prelude::*,
};

use crate::{
    capabilities,
    components::{GunlikeAnimation, PhysicsBody, ThrowableItem},
    nodes::{armed_kickbomb, Player},
    GameWorld, Resources,
};

pub struct Kickbomb {
    sprite: GunlikeAnimation,
    pub amount: i32,
    pub body: PhysicsBody,
    pub throwable: ThrowableItem,
    cooldown: i32,
}

impl Kickbomb {
    pub const COLLIDER_WIDTH: f32 = 32.0;
    pub const COLLIDER_HEIGHT: f32 = 36.0;
    pub const FIRE_INTERVAL: i32 = 50;
    pub const MAXIMUM_AMOUNT: i32 = 3;

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let resources = storage::get::<Resources>();
        let sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                Self::COLLIDER_WIDTH as u32,
                Self::COLLIDER_HEIGHT as u32,
                &[Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                }],
                false,
            ),
            resources.items_textures["kickbomb/bomb"],
            Self::COLLIDER_WIDTH,
        );

        let mut world = storage::get_mut::<GameWorld>();

        scene::add_node(Kickbomb {
            sprite,
            body: PhysicsBody::new(
                &mut world.collision_world,
                pos,
                0.0,
                vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT),
            ),
            amount: Self::MAXIMUM_AMOUNT,
            throwable: ThrowableItem::default(),
            cooldown: 0,
        })
        .untyped()
    }

    fn draw_hud(&self) {
        let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
        let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);
        for i in 0..Self::MAXIMUM_AMOUNT {
            let x = self.body.pos.x + 15.0 * i as f32;

            if i >= self.amount {
                draw_circle_lines(x, self.body.pos.y - 12.0, 4.0, 2., empty_color);
            } else {
                draw_circle(x, self.body.pos.y - 12.0, 4.0, full_color);
            };
        }
    }

    pub fn throw(&mut self, force: bool) {
        self.throwable.throw(&mut self.body, force);
    }

    pub fn shoot(node: Handle<Kickbomb>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let mut node = scene::get_node(node);
                let player = &mut *scene::get_node(player);
                player.state_machine.set_state(Player::ST_NORMAL);

                if node.amount <= 0 || node.cooldown > 0 {
                    return;
                }
                let pos = if node.body.facing {
                    Vec2::new(40., -10.)
                } else {
                    Vec2::new(-20., -10.)
                };
                armed_kickbomb::ArmedKickBomb::spawn(node.body.pos + pos);
                node.amount -= 1;
                node.cooldown = Self::FIRE_INTERVAL;
            }
        };
        start_coroutine(coroutine)
    }
    pub fn weapon_capabilities() -> capabilities::Weapon {
        pub fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool, inverted: bool) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Kickbomb>();
            let mount_pos = if node.body.facing {
                vec2(0., 16.)
            } else {
                vec2(-20., 16.)
            };

            node.body.pos = parent_pos + mount_pos;
            node.body.facing = parent_facing;
            node.body.inverted = inverted;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Kickbomb>();
            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                Kickbomb::COLLIDER_WIDTH,
                Kickbomb::COLLIDER_HEIGHT,
            )
        }

        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Kickbomb>();
            Kickbomb::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Kickbomb>()
                .handle();
            Kickbomb::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Kickbomb>();
            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Kickbomb>();
            node.body.angle = 0.;
            node.amount = Kickbomb::MAXIMUM_AMOUNT;
            node.throwable.owner = Some(owner);
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
                .to_typed::<Kickbomb>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Kickbomb>();

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
                .to_typed::<Kickbomb>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Kickbomb>();
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

impl scene::Node for Kickbomb {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::weapon_capabilities());
        node.provides(Self::physics_capabilities());
    }

    fn draw(node: RefMut<Self>) {
        node.sprite.draw(
            node.body.pos,
            node.body.facing,
            node.body.inverted,
            node.body.angle,
        );
        if !node.throwable.thrown() {
            node.draw_hud();
        }
    }

    fn fixed_update(mut node: RefMut<Self>) {
        node.cooldown -= 1;
        node.sprite.update();
        let node = &mut *node;
        node.throwable.update(&mut node.body, true);
    }
}
