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

use crate::nodes::ParticleEmitters;
use crate::{
    capabilities,
    components::{GunlikeAnimation, PhysicsBody, ThrowableItem},
    nodes::Player,
    GameWorld, Resources,
};

pub struct FlyingGalleon {
    pub sprite: Texture2D,
    pub pos: Vec2,
    pub speed: Vec2,
    pub facing: bool,
    pub owner_id: u8,
}

impl FlyingGalleon {
    pub const SPEED: f32 = 200.;
    pub const WIDTH: f32 = 425.;
    pub const HEIGHT: f32 = 390.;
    pub const KNOCKBACK: f32 = 10.;

    pub fn new(owner_id: u8) -> FlyingGalleon {
        let resources = storage::get::<Resources>();
        let texture_entry = resources.textures.get("galleon").unwrap();

        let (pos, facing) = Self::start_position();
        let dir = if facing {
            Vec2::new(1., 0.)
        } else {
            Vec2::new(-1., 0.)
        };

        FlyingGalleon {
            sprite: texture_entry.texture,
            pos,
            speed: dir * Self::SPEED,
            facing,
            owner_id,
        }
    }

    pub fn start_position() -> (Vec2, bool) {
        let world = storage::get::<GameWorld>();

        let map_size = world.map.get_size();

        let facing = gen_range(0, 2) == 0;
        let pos = Vec2::new(
            if facing { -Self::WIDTH } else { map_size.x },
            gen_range(0., map_size.y - Self::HEIGHT),
        );

        (pos, facing)
    }

    pub fn update(&mut self) -> bool {
        self.pos += self.speed * get_frame_time();

        {
            let world = storage::get::<GameWorld>();

            let map_size = world.map.get_size();

            if self.pos.x + Self::WIDTH < -map_size.x * 0.5 || self.pos.x > map_size.x * 1.5 {
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
                    let mut particles = scene::find_node_by_type::<ParticleEmitters>().unwrap();
                    particles.hit.spawn(player.body.pos);
                }
                let direction = self.pos.x > (player.body.pos.x + Self::KNOCKBACK);
                player.kill(direction);
            }
        }

        true
    }

    pub fn draw(&self, pos: Vec2, facing: bool) {
        draw_texture_ex(
            self.sprite,
            pos.x,
            pos.y,
            color::WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(Self::WIDTH, Self::HEIGHT)),
                flip_x: facing,
                ..Default::default()
            },
        );
    }
}

impl Node for FlyingGalleon {
    fn draw(node: RefMut<Self>) {
        node.draw(node.pos, node.facing);
    }

    fn fixed_update(mut node: RefMut<Self>) {
        if !node.update() {
            node.delete();
        }
    }
}

pub struct Galleon {
    sprite: GunlikeAnimation,

    body: PhysicsBody,
    throwable: ThrowableItem,
}

impl Galleon {
    pub const SPRITE_WIDTH: u32 = 32;
    pub const SPRITE_HEIGHT: u32 = 29;
    pub const RIGHT_OFFSET: [f32; 2] = [15., 16.];
    pub const LEFT_OFFSET: [f32; 2] = [-22., 16.];

    pub fn throw(&mut self, force: bool) {
        self.throwable.throw(&mut self.body, force);
    }

    pub fn shoot(node: Handle<Galleon>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            let player = &mut *scene::get_node(player);
            scene::add_node(FlyingGalleon::new(player.id));

            player.state_machine.set_state(Player::ST_NORMAL);

            player.weapon = None;
            scene::get_node(node).delete();
        };

        start_coroutine(coroutine)
    }

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let resources = storage::get::<Resources>();
        let texture_entry = resources.textures.get("galleon_icon").unwrap();

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
            texture_entry.texture,
            Self::SPRITE_WIDTH as f32,
        );

        let mut world = storage::get_mut::<GameWorld>();

        let body = PhysicsBody::new(
            &mut world.collision_world,
            pos,
            0.0,
            vec2(Self::SPRITE_WIDTH as f32, Self::SPRITE_HEIGHT as f32),
            false,
        );

        scene::add_node(Galleon {
            sprite,
            body,
            throwable: ThrowableItem::default(),
        })
        .untyped()
    }

    pub fn weapon_capabilities() -> capabilities::Weapon {
        fn throw(node: HandleUntyped, force: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Galleon>();

            Galleon::throw(&mut *node, force);
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Galleon>()
                .handle();
            Galleon::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Galleon>();

            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Galleon>();

            node.body.angle = 0.;
            node.throwable.owner = Some(owner);
        }

        fn mount(
            node: HandleUntyped,
            parent_pos: Vec2,
            parent_facing: bool,
            parent_inverted: bool,
        ) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Galleon>();
            let mount_pos = if node.body.facing {
                vec2(Galleon::RIGHT_OFFSET[0], Galleon::RIGHT_OFFSET[1])
            } else {
                vec2(Galleon::LEFT_OFFSET[0], Galleon::LEFT_OFFSET[1])
            };

            node.body.pos = parent_pos + mount_pos;
            node.body.facing = parent_facing;
            node.body.inverted = parent_inverted;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Galleon>();
            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                Galleon::SPRITE_WIDTH as f32,
                Galleon::SPRITE_HEIGHT as f32,
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
                .to_typed::<Galleon>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Galleon>();

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
                .to_typed::<Galleon>();
            node.body.velocity.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Galleon>();
            node.body.velocity.y = speed;
        }

        capabilities::PhysicsObject {
            active,
            collider,
            set_speed_x,
            set_speed_y,
        }
    }
}

impl Node for Galleon {
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
