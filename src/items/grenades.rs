use macroquad::{
    //audio::play_sound_once,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, Handle, HandleUntyped, Node, RefMut},
    },
    prelude::*,
};

use crate::{
    capabilities,
    components::{ArmedGrenade, GunlikeAnimation, PhysicsBody, ThrowableItem},
    nodes::Player,
    Resources,
};

pub struct Grenades {
    grenade_sprite: GunlikeAnimation,

    pub thrown: bool,

    pub amount: i32,
    pub body: PhysicsBody,
    pub throwable: ThrowableItem,
}

impl Grenades {
    pub const COLLIDER_WIDTH: f32 = 21.0;
    pub const COLLIDER_HEIGHT: f32 = 29.0;
    pub const FIRE_INTERVAL: f32 = 0.25;
    pub const MAXIMUM_AMOUNT: i32 = 3;

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let mut resources = storage::get_mut::<Resources>();

        let grenade_sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                21,
                28,
                &[Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                }],
                false,
            ),
            resources.items_textures["grenades/explosives"],
            Self::COLLIDER_WIDTH,
        );

        scene::add_node(Grenades {
            grenade_sprite,
            body: PhysicsBody::new(
                &mut resources.collision_world,
                pos,
                0.,
                vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT),
            ),
            thrown: false,
            throwable: ThrowableItem::default(),
            amount: Self::MAXIMUM_AMOUNT,
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

    pub fn shoot(node: Handle<Grenades>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let mut node = scene::get_node(node);
                if node.amount <= 0 {
                    let player = &mut *scene::get_node(player);
                    player.state_machine.set_state(Player::ST_NORMAL);

                    return;
                }

                ArmedGrenade::spawn(node.body.pos, node.body.facing);
                node.amount -= 1;
            }

            wait_seconds(Grenades::FIRE_INTERVAL).await;

            {
                let player = &mut *scene::get_node(player);
                player.state_machine.set_state(Player::ST_NORMAL);
            }
        };

        start_coroutine(coroutine)
    }

    pub fn weapon_capabilities() -> capabilities::Weapon {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Grenades>();

            Grenades::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Grenades>()
                .handle();

            Grenades::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Grenades>();

            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Grenades>();

            node.body.angle = 0.;
            node.amount = Grenades::MAXIMUM_AMOUNT;

            node.thrown = false;

            node.throwable.owner = Some(owner);
        }

        fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Grenades>();
            let mount_pos = if node.body.facing {
                vec2(0., 16.)
            } else {
                vec2(-20., 16.)
            };

            node.body.pos = parent_pos + mount_pos;
            node.body.facing = parent_facing;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Grenades>();
            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                Grenades::COLLIDER_WIDTH,
                Grenades::COLLIDER_HEIGHT,
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
                .to_typed::<Grenades>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Grenades>();

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
                .to_typed::<Grenades>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Grenades>();
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

impl Node for Grenades {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::weapon_capabilities());
        node.provides(Self::physics_capabilities());
    }

    fn fixed_update(mut node: RefMut<Self>) {
        let node = &mut *node;

        node.grenade_sprite.update();
        node.throwable.update(&mut node.body, true);
    }

    fn draw(node: RefMut<Self>) {
        node.grenade_sprite
            .draw(node.body.pos, node.body.facing, node.body.angle);

        if !node.throwable.thrown() {
            node.draw_hud();
        }
    }
}
