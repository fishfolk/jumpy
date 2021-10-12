use macroquad::{
    math::Rect,
    prelude::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        draw_circle, draw_circle_lines, get_frame_time,
        scene::{self, Handle, HandleUntyped, RefMut},
        vec2, Color, Vec2,
    },
};

use crate::{
    capabilities,
    components::{GunlikeAnimation, PhysicsBody, ThrowableItem},
    nodes::{explosive, Player},
    GameWorld, Resources,
};

pub struct Cannon {
    cannon_sprite: GunlikeAnimation,

    pub amount: i32,

    pub body: PhysicsBody,
    pub throwable: ThrowableItem,

    grace_time: f32,
}

impl Cannon {
    const INITIAL_CANNONBALLS: i32 = 3;
    const MAXIMUM_CANNONBALLS: i32 = 3;

    const CANNON_WIDTH: f32 = 88.;
    const CANNON_HEIGHT: f32 = 36.;
    const CANNON_ANIMATION_BASE: &'static str = "base";
    const CANNON_ANIMATION_SHOOT: &'static str = "shoot";

    const CANNON_THROWBACK: f32 = 1050.0;
    const SHOOTING_GRACE_TIME: f32 = 1.0; // seconds

    // --- Cannonball Settings
    const CANNONBALL_TEXTURE_ID: &'static str = "cannonball";
    const CANNONBALL_COUNTDOWN_DURATION: f32 = 0.5;
    /// After shooting, the owner is safe for this amount of time. This is crucial, otherwise, given the
    /// large hitbox, they will die immediately on shoot.
    /// The formula is simplified (it doesn't include mount position, run speed and throwback).
    const CANNONBALL_OWNER_SAFE_TIME: f32 =
        Self::EXPLOSION_RADIUS / Self::CANNONBALL_INITIAL_SPEED_X_REL;

    const CANNONBALL_WIDTH: f32 = 32.;
    pub const CANNONBALL_HEIGHT: f32 = 36.;
    const CANNONBALL_ANIMATION_ROLLING: &'static str = "rolling";
    const CANNONBALL_INITIAL_SPEED_X_REL: f32 = 600.;
    const CANNONBALL_INITIAL_SPEED_Y: f32 = -200.;

    const EXPLOSION_RADIUS: f32 = 4. * Self::CANNONBALL_WIDTH;

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let resources = storage::get::<Resources>();
        let texture_entry = resources.textures.get("cannon").unwrap();

        let cannon_sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                Self::CANNON_WIDTH as u32,
                Self::CANNON_HEIGHT as u32,
                &[
                    Animation {
                        name: Self::CANNON_ANIMATION_BASE.to_string(),
                        row: 0,
                        frames: 1,
                        fps: 1,
                    },
                    Animation {
                        name: Self::CANNON_ANIMATION_SHOOT.to_string(),
                        row: 1,
                        frames: 4,
                        fps: 8,
                    },
                ],
                false,
            ),
            texture_entry.texture,
            Self::CANNON_WIDTH,
        );

        let mut world = storage::get_mut::<GameWorld>();

        scene::add_node(Self {
            cannon_sprite,
            body: PhysicsBody::new(
                &mut world.collision_world,
                pos,
                0.0,
                vec2(Self::CANNON_WIDTH, Self::CANNON_HEIGHT),
            ),
            throwable: ThrowableItem::default(),
            amount: Self::INITIAL_CANNONBALLS,
            grace_time: 0.,
        })
        .untyped()
    }

    fn draw_hud(&self) {
        let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
        let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);
        for i in 0..Self::MAXIMUM_CANNONBALLS {
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

    pub fn shoot(node_h: Handle<Cannon>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                scene::find_node_by_type::<crate::nodes::Camera>()
                    .unwrap()
                    .shake_noise_dir(0.8, 4, 0.2, (1.0, 0.1));
            }
            {
                let mut node = scene::get_node(node_h);

                if node.amount <= 0 || node.grace_time > 0. {
                    let player: &mut Player = &mut *scene::get_node(player);
                    player.state_machine.set_state(Player::ST_NORMAL);

                    node.grace_time -= get_frame_time();

                    return;
                } else {
                    node.grace_time = Self::SHOOTING_GRACE_TIME;
                }

                let player: &mut Player = &mut *scene::get_node(player);

                let cannonball_pos = vec2(
                    node.body.pos.x,
                    node.body.pos.y + 20. - (Self::CANNONBALL_HEIGHT as f32 / 2.),
                );

                explosive::Explosive::spawn(
                    cannonball_pos,
                    vec2(
                        Self::CANNONBALL_INITIAL_SPEED_X_REL * player.body.facing_dir().x,
                        Self::CANNONBALL_INITIAL_SPEED_Y,
                    ),
                    explosive::DetonationParameters {
                        trigger_radius: Some(Self::EXPLOSION_RADIUS),
                        owner_safe_fuse: Self::CANNONBALL_OWNER_SAFE_TIME,
                        explosion_radius: Self::EXPLOSION_RADIUS,
                        fuse: Some(Self::CANNONBALL_COUNTDOWN_DURATION),
                    },
                    Self::CANNONBALL_TEXTURE_ID,
                    AnimatedSprite::new(
                        Self::CANNONBALL_WIDTH as u32,
                        Self::CANNONBALL_HEIGHT as u32,
                        &[Animation {
                            name: Self::CANNONBALL_ANIMATION_ROLLING.to_string(),
                            row: 0,
                            frames: 1,
                            fps: 1,
                        }],
                        true,
                    ),
                    vec2(Self::CANNONBALL_WIDTH, Self::CANNONBALL_HEIGHT),
                    player.id,
                );

                player.body.speed.x = -Self::CANNON_THROWBACK * player.body.facing_dir().x;
            }

            wait_seconds(0.08).await;

            {
                {
                    let mut node = scene::get_node(node_h);
                    node.amount -= 1;
                }

                {
                    let player: &mut Player = &mut *scene::get_node(player);
                    player.state_machine.set_state(Player::ST_NORMAL);
                }
                {
                    let mut node = scene::get_node(node_h);
                    node.cannon_sprite.set_animation(1);
                }
                for i in 0u32..4 {
                    {
                        let mut node = scene::get_node(node_h);
                        node.cannon_sprite.set_frame(i);
                    }
                    wait_seconds(0.08).await;
                }
                {
                    let mut node = scene::get_node(node_h);
                    node.cannon_sprite.set_animation(0);
                }
            }
        };

        start_coroutine(coroutine)
    }

    fn physics_capabilities() -> capabilities::PhysicsObject {
        fn active(handle: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Cannon>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Cannon>();

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
                .to_typed::<Cannon>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Cannon>();
            node.body.speed.y = speed;
        }

        capabilities::PhysicsObject {
            active,
            collider,
            set_speed_x,
            set_speed_y,
        }
    }

    pub fn weapon_capabilities() -> capabilities::Weapon {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Cannon>();

            Cannon::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Cannon>()
                .handle();

            Cannon::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Cannon>();

            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Cannon>();

            node.body.angle = 0.;
            node.amount = Cannon::INITIAL_CANNONBALLS;
            node.throwable.owner = Some(owner);
        }

        fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Cannon>();
            let mount_pos = if node.body.facing {
                vec2(0., 16.)
            } else {
                vec2(-60., 16.)
            };

            node.body.pos = parent_pos + mount_pos;
            node.body.facing = parent_facing;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Cannon>();
            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                Cannon::CANNON_WIDTH,
                Cannon::CANNON_HEIGHT,
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
}

impl scene::Node for Cannon {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::weapon_capabilities());
        node.provides(Self::physics_capabilities());
    }

    fn fixed_update(mut node: RefMut<Self>) {
        let node = &mut *node;

        node.cannon_sprite.update();
        node.throwable.update(&mut node.body, true);

        node.grace_time -= get_frame_time();
    }

    fn draw(node: RefMut<Self>) {
        node.cannon_sprite
            .draw(node.body.pos, node.body.facing, node.body.angle);

        if !node.throwable.thrown() {
            node.draw_hud();
        }
    }
}
