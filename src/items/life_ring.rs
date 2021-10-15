use macroquad::{
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::{HandleUntyped, Handle, Node, RefMut},
        coroutines::{start_coroutine, Coroutine},
    },
    prelude::*,
};

use crate::{
    nodes::Player,
    capabilities,
    Resources, GameWorld,
    components::{PhysicsBody, GunlikeAnimation, ThrowableItem}
};

use std::f32;
pub struct LifeRing {
    sprite: GunlikeAnimation,
    used: bool,
    pub body: PhysicsBody,
    pub throwable: ThrowableItem,
    pub mount_pos: [Vec2; 2],
}

impl LifeRing {
    const COLLIDER_WIDTH: f32 = 44.0;
    const COLLIDER_HEIGHT: f32 = 12.0;

    pub const PICK_MOUNT: [(f32, f32); 2] = [(35.0, 0.0), (-44.0, 0.0)];
    pub const EQUIP_MOUNT: [(f32, f32); 2] = [(0.0, 0.0), (-10.0, 0.0)];


    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let resources = storage::get_mut::<Resources>();


        let sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                Self::COLLIDER_WIDTH as u32,
                Self::COLLIDER_HEIGHT as u32,
                &[
                    Animation {
                        name: "idle".to_string(),
                        row: 0,
                        frames: 1,
                        fps: 1,
                    }
                ],
                false,
            ),
            resources.items_textures["life_ring/life_ring"],
            Self::COLLIDER_WIDTH,
        );

        let mut world = storage::get_mut::<GameWorld>();


        scene::add_node(LifeRing {
            sprite,
            used: false,
            body: PhysicsBody::new(
                &mut world.collision_world,
                pos,
                0.0,
                vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT)
            ),
            throwable: ThrowableItem::default(),
            mount_pos: [
                vec2(Self::PICK_MOUNT[0].0, Self::PICK_MOUNT[0].1),
                vec2(Self::PICK_MOUNT[1].0, Self::PICK_MOUNT[1].1)
            ],
        }).untyped()
    }

    pub fn throw(&mut self, force: bool) {
        self.throwable.throw(&mut self.body, force);
    }

    pub fn shoot(node: Handle<LifeRing>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            let life_ring = &mut *scene::get_node(node);
            life_ring.mount_pos = [
                vec2(Self::EQUIP_MOUNT[0].0, Self::EQUIP_MOUNT[0].1),
                vec2(Self::EQUIP_MOUNT[1].0, Self::EQUIP_MOUNT[1].1)
            ];

            {
                let player = &mut *scene::get_node(player);
                player.body.gravity_dir = -1.4;
                player.body.inverted = true;
                player.body.speed.y = -600.0;
            }

            life_ring.used = true;
            
            let player = &mut *scene::get_node(player);
            player.state_machine.set_state(Player::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }

    pub fn weapon_capabilities() -> capabilities::Weapon {
        fn throw(node: HandleUntyped, force: bool) {
            if let Some(life_ring) = scene::get_untyped_node(node) {
                LifeRing::throw(&mut *life_ring.to_typed::<LifeRing>(), force);
            }
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine{
            if let Some(life_ring) = scene::get_untyped_node(node) {
                let life_ring = life_ring.to_typed::<LifeRing>().handle();
                LifeRing::shoot(life_ring, player)
            } else {
                let coroutine = async move {
                    let player = &mut *scene::get_node(player);
                    player.state_machine.set_state(Player::ST_NORMAL);
                };
                start_coroutine(coroutine)
            }
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            if let Some(life_ring) = scene::get_untyped_node(node) {
                life_ring.to_typed::<LifeRing>().throwable.thrown()
            } else {
                false
            }
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            if let Some(life_ring) = scene::get_untyped_node(node) {
                let mut life_ring = life_ring.to_typed::<LifeRing>();
                life_ring.body.angle = 0.;
                life_ring.throwable.owner = Some(owner);
            }
        }

        fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool, parent_inverted: bool) {
            if let Some(life_ring) = scene::get_untyped_node(node) {
                let mut life_ring = life_ring.to_typed::<LifeRing>();
                let mount_pos = if life_ring.body.facing {
                    life_ring.mount_pos[0]
                } else {
                    life_ring.mount_pos[1]
                };
    
                life_ring.body.pos = parent_pos + mount_pos;
                life_ring.body.facing = parent_facing;
                life_ring.body.inverted = parent_inverted;
            }
        }

        fn collider(node: HandleUntyped) -> Rect {
            match scene::get_untyped_node(node) {
                Some(life_ring) => {
                    let life_ring = life_ring.to_typed::<LifeRing>();
                    return Rect::new(
                        life_ring.body.pos.x,
                        life_ring.body.pos.y,
                        LifeRing::COLLIDER_WIDTH as f32,
                        LifeRing::COLLIDER_HEIGHT as f32,
                    )
                }
                None => return Rect::new(
                    f32::MAX,
                    f32::MAX,
                    0.0,
                    0.0,
                ),
            }
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
                .to_typed::<LifeRing>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<LifeRing>();

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
                .to_typed::<LifeRing>();
            node.body.speed.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<LifeRing>();
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

impl Node for LifeRing {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::weapon_capabilities());
        node.provides(Self::physics_capabilities());
    }

    fn fixed_update(mut node: RefMut<Self>) {
        {
            let node = &mut *node;
            node.sprite.update();
            node.throwable.update(&mut node.body, true);
        }
        if node.used {
            node.delete();
        }
    }
    

    fn draw(node: RefMut<Self>) {
        node.sprite.draw(node.body.pos, node.body.facing, node.body.inverted, node.body.angle);
    }
}
