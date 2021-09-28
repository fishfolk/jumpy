use macroquad::{
    audio::play_sound_once,
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
    components::{Bullet, GunlikeAnimation, PhysicsBody, ThrowableItem},
    nodes::Player,
    Resources,
};

pub struct Gun {
    pub gun_sprite: GunlikeAnimation,
    pub gun_fx_sprite: GunlikeAnimation,
    pub gun_fx: bool,

    pub bullets: i32,

    pub body: PhysicsBody,
    pub throwable: ThrowableItem,

    pub collider_width: f32,
    pub collider_height: f32,
    pub recoil: f32,
}

impl Gun {
    fn draw_hud(&self) {
        let full_color = Color::new(0.8, 0.9, 1.0, 1.0);
        let empty_color = Color::new(0.8, 0.9, 1.0, 0.8);
        for i in 0..3 {
            let x = self.body.pos.x + 15.0 * i as f32;

            if i >= self.bullets {
                draw_circle_lines(x, self.body.pos.y - 12.0, 4.0, 2., empty_color);
            } else {
                draw_circle(x, self.body.pos.y - 12.0, 4.0, full_color);
            };
        }
    }

    pub fn throw(&mut self, force: bool) {
        self.throwable.throw(&mut self.body, force);
    }

    pub fn shoot(node: Handle<Gun>, player: Handle<Player>) -> Coroutine {
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

                let mut node = &mut *scene::get_node(node);
                let player = &mut *scene::get_node(player);

                node.gun_fx = true;

                scene::add_node(GunBullet::new(
                    node.body.pos + vec2(16.0, 15.0) + node.body.facing_dir() * 32.0,
                    node.body.facing,
                    4.,
                ));
                player.body.speed.x = -node.recoil * player.body.facing_dir().x;
            }
            {
                let node = &mut *scene::get_node(node);
                node.gun_sprite.set_animation(1);
            }
            for i in 0u32..3 {
                {
                    let node = &mut *scene::get_node(node);
                    node.gun_sprite.set_frame(i);
                    node.gun_fx_sprite.set_frame(i);
                }

                wait_seconds(0.08).await;
            }
            {
                let mut node = scene::get_node(node);
                node.gun_sprite.set_animation(0);
            }

            {
                let mut node = scene::get_node(node);
                node.gun_fx = false;
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
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Gun>();

            Gun::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Gun>()
                .handle();

            Gun::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Gun>();

            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Gun>();

            node.body.angle = 0.;
            node.bullets = 3;
            node.throwable.owner = Some(owner);
        }

        fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Gun>();
            let mount_pos = if node.body.facing {
                vec2(0., 16.)
            } else {
                vec2(-20., 16.)
            };

            node.body.pos = parent_pos + mount_pos;
            node.body.facing = parent_facing;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Gun>();
            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                node.collider_width,
                node.collider_height,
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
            let node = scene::get_untyped_node(handle).unwrap().to_typed::<Gun>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle).unwrap().to_typed::<Gun>();

            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                node.body.size.x,
                node.body.size.y,
            )
        }
        fn set_speed_x(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle).unwrap().to_typed::<Gun>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle).unwrap().to_typed::<Gun>();
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

impl scene::Node for Gun {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::weapon_capabilities());
        node.provides(Self::physics_capabilities());
    }

    fn draw(node: RefMut<Self>) {
        node.gun_sprite
            .draw(node.body.pos, node.body.facing, node.body.angle);

        if node.gun_fx {
            node.gun_fx_sprite
                .draw(node.body.pos, node.body.facing, node.body.angle);
        }

        if !node.throwable.thrown() {
            node.draw_hud();
        }
    }

    fn fixed_update(mut node: RefMut<Self>) {
        let node = &mut *node;

        node.gun_sprite.update();
        node.gun_fx_sprite.update();
        node.throwable.update(&mut node.body, true);
    }
}

pub struct GunBullet {
    bullet: Bullet,
    size: f32,
}

impl GunBullet {
    pub const BULLET_SPEED: f32 = 500.0;
    pub const BULLET_LIFETIME: f32 = 0.9;
    pub const BULLET_SPREAD: f32 = 0.0;

    pub fn new(pos: Vec2, facing: bool, size: f32) -> GunBullet {
        GunBullet {
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

impl GunBullet {
    fn network_capabilities() -> capabilities::NetworkReplicate {
        fn network_update(handle: HandleUntyped) {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<GunBullet>();
            GunBullet::network_update(node);
        }

        capabilities::NetworkReplicate { network_update }
    }

    fn network_update(mut node: RefMut<Self>) {
        if !node.bullet.update() {
            node.delete();
        }
    }
}
impl scene::Node for GunBullet {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::network_capabilities());
    }

    fn draw(node: RefMut<Self>) {
        draw_circle(
            node.bullet.pos.x,
            node.bullet.pos.y,
            node.size,
            Color::new(1.0, 1.0, 0.8, 1.0),
        );
    }
}
