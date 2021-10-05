use macroquad::{
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, Handle, HandleUntyped, RefMut},
    },
    prelude::*,
};

use crate::{
    capabilities,
    components::{GunlikeAnimation, PhysicsBody, ThrowableItem},
    nodes::{explosive, Player},
    GameWorld, Resources,
};

pub struct Mines {
    mines_sprite: GunlikeAnimation,
    pub amount: i32,
    pub body: PhysicsBody,
    pub throwable: ThrowableItem,
}

impl Mines {
    pub const COLLIDER_WIDTH: f32 = 32.0;
    pub const COLLIDER_HEIGHT: f32 = 16.0;
    pub const FIRE_INTERVAL: f32 = 0.5;
    pub const MAXIMUM_AMOUNT: i32 = 3;

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let resources = storage::get::<Resources>();
        let mines_sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                26,
                40,
                &[Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                }],
                false,
            ),
            resources.items_textures["mines/mines"],
            Self::COLLIDER_WIDTH,
        );

        let mut world = storage::get_mut::<GameWorld>();

        scene::add_node(Mines {
            mines_sprite,
            body: PhysicsBody::new(
                &mut world.collision_world,
                pos,
                0.0,
                vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT),
            ),
            amount: Self::MAXIMUM_AMOUNT,
            throwable: ThrowableItem::default(),
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

    pub fn shoot(node: Handle<Mines>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let mut node = scene::get_node(node);
                if node.amount <= 0 {
                    let player = &mut *scene::get_node(player);
                    player.state_machine.set_state(Player::ST_NORMAL);
                    return;
                }
                let player = &mut *scene::get_node(player);
                explosive::Explosive::spawn(
                    node.body.pos,
                    vec2(0., 0.),
                    explosive::DetonationParameters {
                        trigger_radius: Some(30.),
                        owner_safe_fuse: 1.,
                        explosion_radius: 30.,
                        fuse: None,
                    },
                    "mines/mines",
                    AnimatedSprite::new(
                        32,
                        32,
                        &[Animation {
                            name: "armed".to_string(),
                            row: 0,
                            frames: 2,
                            fps: 3,
                        }],
                        true,
                    ),
                    vec2(32., 32.),
                    player.id,
                );
                node.amount -= 1;
            }
            wait_seconds(Mines::FIRE_INTERVAL).await;
            {
                let player = &mut *scene::get_node(player);
                player.state_machine.set_state(Player::ST_NORMAL);
            }
        };
        start_coroutine(coroutine)
    }

    pub fn weapon_capabilities() -> capabilities::Weapon {
        fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Mines>();
            let mount_pos = if node.body.facing {
                vec2(0., 16.)
            } else {
                vec2(-20., 16.)
            };

            node.body.pos = parent_pos + mount_pos;
            node.body.facing = parent_facing;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Mines>();
            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                Mines::COLLIDER_WIDTH,
                Mines::COLLIDER_HEIGHT,
            )
        }

        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Mines>();
            Mines::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Mines>()
                .handle();
            Mines::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Mines>();
            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Mines>();
            node.body.angle = 0.;
            node.amount = Mines::MAXIMUM_AMOUNT;
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
            let node = scene::get_untyped_node(handle).unwrap().to_typed::<Mines>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle).unwrap().to_typed::<Mines>();

            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                node.body.size.x,
                node.body.size.y,
            )
        }
        fn set_speed_x(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle).unwrap().to_typed::<Mines>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle).unwrap().to_typed::<Mines>();
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

impl scene::Node for Mines {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::weapon_capabilities());
        node.provides(Self::physics_capabilities());
    }

    fn draw(node: RefMut<Self>) {
        node.mines_sprite
            .draw(node.body.pos, node.body.facing, node.body.angle);
        if !node.throwable.thrown() {
            node.draw_hud();
        }
    }

    fn fixed_update(mut node: RefMut<Self>) {
        let node = &mut *node;
        node.mines_sprite.update();
        node.throwable.update(&mut node.body, true);
    }
}
