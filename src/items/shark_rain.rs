use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, Coroutine},
        scene::{Handle, HandleUntyped, Node, RefMut},
    },
    prelude::*,
    rand::gen_range,
};

use crate::{
    capabilities,
    components::{GunlikeAnimation, PhysicsBody, ThrowableItem},
    nodes::Player,
    GameWorld, Resources,
};

pub struct RainingShark {
    pub sprite: Texture2D,
    pub pos: Vec2,
    pub speed: Vec2,
    pub owner_id: u8,
}

impl RainingShark {
    pub const SPEED: f32 = 500.;
    pub const WIDTH: f32 = 60.;
    pub const HEIGHT: f32 = 220.;
    pub const KNOCKBACK: f32 = 10.;
    pub const COUNT: usize = 5;
    pub const START_HEIGHT: f32 = -100.;
    pub const HEIGHT_DIFF: f32 = 200.;

    pub fn new(owner_id: u8, pos: Vec2) -> RainingShark {
        let resources = storage::get::<Resources>();
        let sprite = resources.items_textures["shark_rain/raining_shark"];

        RainingShark {
            sprite,
            pos,
            speed: Vec2::new(0., Self::SPEED),
            owner_id,
        }
    }

    pub fn update(&mut self) -> bool {
        self.pos += self.speed * get_frame_time();

        {
            let world = storage::get::<GameWorld>();
            let map_size = world.map.get_size();
            if self.pos.y > map_size.y {
                return false;
            }
        }

        for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
            if player.get_hitbox().overlaps(&Rect::new(
                self.pos.x,
                self.pos.y,
                Self::WIDTH,
                Self::HEIGHT,
            )) && player.id != self.owner_id
                && !player.dead
            {
                scene::find_node_by_type::<crate::nodes::Camera>()
                    .unwrap()
                    .shake_noise(1.0, 10, 1.);
                {
                    let mut resources = storage::get_mut::<Resources>();
                    resources.hit_fxses.spawn(player.body.pos);
                }
                let direction = self.pos.x > (player.body.pos.x + Self::KNOCKBACK);
                player.kill(direction);
            }
        }

        true
    }

    pub fn draw(&self, pos: Vec2) {
        draw_texture_ex(
            self.sprite,
            pos.x,
            pos.y,
            color::WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(Self::WIDTH, Self::HEIGHT)),
                ..Default::default()
            },
        );
    }
}

impl Node for RainingShark {
    fn draw(node: RefMut<Self>) {
        node.draw(node.pos);
    }

    fn fixed_update(mut node: RefMut<Self>) {
        if !node.update() {
            node.delete();
        }
    }
}

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

            let positions = Self::start_positions();

            for pos in positions {
                scene::add_node(RainingShark::new(player.id, pos));
            }

            player.weapon = None;
            scene::get_node(node).delete();
        };

        start_coroutine(coroutine)
    }

    pub fn start_positions() -> Vec<Vec2> {
        let world = storage::get::<GameWorld>();
        let map_size = world.map.get_size();

        let mut positions: Vec<Vec2> = Vec::new();
        let mut start = 0.;
        let mut quant = RainingShark::COUNT as f32;

        for _ in 0..RainingShark::COUNT {
            let free_space = (map_size.x - start - RainingShark::WIDTH * quant) / quant;
            let pos = Vec2::new(
                gen_range(start, free_space + start),
                gen_range(
                    RainingShark::START_HEIGHT,
                    RainingShark::START_HEIGHT - RainingShark::HEIGHT_DIFF,
                ),
            );

            positions.push(pos);
            quant -= 1.;
            start = pos.x + RainingShark::WIDTH;
        }

        positions
    }

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let resources = storage::get::<Resources>();

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

        let mut world = storage::get_mut::<GameWorld>();

        let body = PhysicsBody::new(
            &mut world.collision_world,
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

        fn mount(
            node: HandleUntyped,
            parent_pos: Vec2,
            parent_facing: bool,
            parent_inverted: bool,
        ) {
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
            node.body.inverted = parent_inverted;
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
        node.sprite.draw(
            node.body.pos,
            node.body.facing,
            node.body.inverted,
            node.body.angle,
        );
    }
}
