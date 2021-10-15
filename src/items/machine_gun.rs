use macroquad::{
    audio::play_sound_once,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{Handle, HandleUntyped, Node, RefMut},
    },
    prelude::*,
};

use crate::{
    capabilities,
    components::{Bullet, GunlikeAnimation, PhysicsBody, ThrowableItem},
    nodes::Player,
    GameWorld, Resources,
};

pub struct MachinegunBullet {
    bullet: Bullet,
    size: f32,
}

impl MachinegunBullet {
    pub const BULLET_SPEED: f32 = 500.0;
    pub const BULLET_LIFETIME: f32 = 0.9;
    pub const BULLET_SPREAD: f32 = 0.1;

    pub fn new(pos: Vec2, facing: bool, size: f32) -> MachinegunBullet {
        MachinegunBullet {
            bullet: Bullet::new(
                pos,
                Self::BULLET_LIFETIME,
                facing,
                Self::BULLET_SPEED,
                Self::BULLET_SPREAD,
            ),
            size,
        }
    }
}
impl scene::Node for MachinegunBullet {
    fn draw(node: RefMut<Self>) {
        draw_circle(
            node.bullet.pos.x,
            node.bullet.pos.y,
            node.size,
            Color::new(1.0, 1.0, 0.8, 1.0),
        );
    }

    fn fixed_update(mut node: RefMut<Self>) {
        if !node.bullet.update() {
            node.delete();
        }
    }
}

pub struct MachineGun {
    sprite: GunlikeAnimation,

    body: PhysicsBody,
    throwable: ThrowableItem,

    pub bullets: i32,
}

impl MachineGun {
    pub const COLLIDER_WIDTH: f32 = 60.0;
    pub const COLLIDER_HEIGHT: f32 = 20.0;

    pub const GUN_THROWBACK: f32 = 75.0;
    pub const FIRE_INTERVAL: f32 = 0.0025; // Time in animation lock between bullets
    pub const MAX_BULLETS: i32 = 20;

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let resources = storage::get::<Resources>();

        let sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                80,
                24,
                &[
                    Animation {
                        name: "idle".to_string(),
                        row: 0,
                        frames: 1,
                        fps: 1,
                    },
                    Animation {
                        name: "shoot".to_string(),
                        row: 1,
                        frames: 2,
                        fps: 8,
                    },
                ],
                false,
            ),
            resources.items_textures["machine_gun/gun"],
            Self::COLLIDER_WIDTH,
        );

        let mut world = storage::get_mut::<GameWorld>();

        let body = PhysicsBody::new(
            &mut world.collision_world,
            pos,
            0.0,
            vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT),
        );

        scene::add_node(MachineGun {
            sprite,
            body,
            throwable: ThrowableItem::default(),
            bullets: Self::MAX_BULLETS,
        })
        .untyped()
    }

    fn draw_hud(&self) {
        let line_height = 16.0;
        let line_spacing = 1.0;
        let line_thickness = 2.0;
        let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
        let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);
        for i in 0..Self::MAX_BULLETS {
            let x = self.body.pos.x - 15.0 + (line_thickness + line_spacing) * i as f32;
            let y = self.body.pos.y - 12.0;

            if i >= self.bullets {
                draw_line(x, y, x, y - line_height, line_thickness, full_color);
            } else {
                draw_line(x, y, x, y - line_height, line_thickness, empty_color);
            };
        }
    }

    pub fn throw(&mut self, force: bool) {
        self.throwable.throw(&mut self.body, force);
    }

    pub fn shoot(node: Handle<MachineGun>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            {
                let node = scene::get_node(node);
                if node.bullets <= 0 {
                    let player = &mut *scene::get_node(player);
                    player.state_machine.set_state(Player::ST_NORMAL);

                    return;
                }
            }

            // {
            //     scene::find_node_by_type::<crate::nodes::Camera>()
            //         .unwrap()
            //         .shake();
            // }
            {
                let resources = storage::get_mut::<Resources>();
                play_sound_once(resources.shoot_sound);

                let node = &mut *scene::get_node(node);
                let player = &mut *scene::get_node(player);

                scene::add_node(MachinegunBullet::new(
                    node.body.pos + vec2(16.0, 6.0) + node.body.facing_dir() * 55.0,
                    node.body.facing,
                    3.,
                ));

                player.body.speed.x = -Self::GUN_THROWBACK * player.body.facing_dir().x;
            }
            {
                let node = &mut *scene::get_node(node);
                node.sprite.set_animation(1);
            }
            for i in 0u32..3 {
                {
                    let node = &mut *scene::get_node(node);
                    node.sprite.set_frame(i);
                }

                wait_seconds(MachineGun::FIRE_INTERVAL / 3.0).await;
            }
            {
                let mut node = scene::get_node(node);
                node.sprite.set_animation(0);
            }

            {
                let mut node = scene::get_node(node);
                node.bullets -= 1;
            }

            {
                let player = &mut *scene::get_node(player);
                // node.weapon_animation.play(0, 0..5).await;
                // node.weapon_animation.play(0, 5..).await;
                player.state_machine.set_state(Player::ST_NORMAL);
            }
        };

        start_coroutine(coroutine)
    }

    pub fn weapon_capabilities() -> capabilities::Weapon {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<MachineGun>();

            MachineGun::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<MachineGun>()
                .handle();

            MachineGun::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<MachineGun>();

            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<MachineGun>();

            node.body.angle = 0.;
            node.bullets = MachineGun::MAX_BULLETS;
            node.throwable.owner = Some(owner);
        }

        fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool, parent_inverted: bool) {
            let mut node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<MachineGun>();
            let mount_pos = if node.body.facing {
                vec2(5., 16.)
            } else {
                vec2(-40., 16.)
            };

            node.body.pos = parent_pos + mount_pos;
            node.body.facing = parent_facing;
            node.body.inverted = parent_inverted;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<MachineGun>();
            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                MachineGun::COLLIDER_WIDTH,
                MachineGun::COLLIDER_HEIGHT,
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
                .to_typed::<MachineGun>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<MachineGun>();

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
                .to_typed::<MachineGun>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<MachineGun>();
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

impl Node for MachineGun {
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
            .draw(node.body.pos, node.body.facing, node.body.inverted, node.body.angle);

        if !node.throwable.thrown() {
            node.draw_hud();
        }
    }
}
