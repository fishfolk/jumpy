use macroquad::{
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{HandleUntyped, Handle, Node, RefMut},
    },
    prelude::*,
};

use crate::{
    nodes::Player,
    capabilities,
    Resources, GameWorld,
    components::{PhysicsBody, GunlikeAnimation, ThrowableItem}
};

pub struct LifeRing {
    sprite: GunlikeAnimation,
    pos: Vec2,
    pub body: PhysicsBody,
    pub throwable: ThrowableItem,
    pub mount_pos: [Vec2; 2],
}

impl LifeRing {
    const COLLIDER_WIDTH: f32 = 44.0;
    const COLLIDER_HEIGHT: f32 = 12.0;

    pub const PICK_MOUNT: [(f32, f32); 2] = [(44.0, 0.0), (-44.0, 0.0)];
    pub const EQUIP_MOUNT: [(f32, f32); 2] = [(0.0, 0.0), (0.0, 0.0)];


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
            pos,
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

    pub fn shoot(node: Handle<LifeRing>, player: Handle<Player>) -> Coroutine{
        let coroutine = async move {
            let life_ring = &mut *scene::get_node(node);
            life_ring.mount_pos = [
                vec2(Self::EQUIP_MOUNT[0].0, Self::EQUIP_MOUNT[0].1),
                vec2(Self::EQUIP_MOUNT[1].0, Self::EQUIP_MOUNT[1].1)
            ];

            {
                let player = &mut *scene::get_node(player);
                player.body.gravity_dir = -1.0;
                //player.body.angle = 180.0;
                player.body.inverted = true;
            }

            wait_seconds(0.08);
            
            let player = &mut *scene::get_node(player);
            player.state_machine.set_state(Player::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }

    pub fn weapon_capabilities() -> capabilities::Weapon {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<LifeRing>();

            LifeRing::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<LifeRing>()
                .handle();
            LifeRing::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<LifeRing>();

            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<LifeRing>();

            node.body.angle = 0.;
            node.throwable.owner = Some(owner);
        }

        fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<LifeRing>();
            let mount_pos = if node.body.facing {
                node.mount_pos[0]
            } else {
                node.mount_pos[1]
            };

            node.body.pos = parent_pos + mount_pos;
            node.body.facing = parent_facing;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<LifeRing>();
            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                LifeRing::COLLIDER_WIDTH as f32,
                LifeRing::COLLIDER_HEIGHT as f32,
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
        let node = &mut *node;
        node.sprite.update();
        node.throwable.update(&mut node.body, true);
    }
    

    fn draw(mut node: RefMut<Self>) {
        node.sprite.draw(node.body.pos, node.body.facing, node.body.angle);
    }


}
