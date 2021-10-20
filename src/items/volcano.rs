use macroquad::{
    color,
    prelude::*,
    prelude::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, Coroutine},
        scene::{Handle, HandleUntyped, Node, RefMut},
        vec2, DrawTextureParams, Vec2,
    },
    rand::gen_range,
};

use crate::{GameWorld, Resources};

use crate::{
    capabilities,
    components::{ArmedGrenade, EruptedItem, GunlikeAnimation, PhysicsBody, ThrowableItem},
    nodes::Player,
};

pub struct Volcano {
    pub sprite: GunlikeAnimation,

    pub throwable: ThrowableItem,
    pub used: bool,

    pub body: PhysicsBody,
}

impl Volcano {
    const COLLIDER_WIDTH: f32 = 36.;
    const COLLIDER_HEIGHT: f32 = 22.;

    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let resources = storage::get::<Resources>();
        let texture_entry = resources.textures.get("volcano_icon").unwrap();

        let sprite = GunlikeAnimation::new(
            AnimatedSprite::new(
                36,
                22,
                &[Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                }],
                false,
            ),
            texture_entry.texture,
            Self::COLLIDER_WIDTH,
        );

        let mut world = storage::get_mut::<GameWorld>();

        let body = PhysicsBody::new(
            &mut world.collision_world,
            pos,
            0.0,
            vec2(Self::COLLIDER_WIDTH, Self::COLLIDER_HEIGHT),
            false,
        );

        scene::add_node(Volcano {
            sprite,
            body,
            throwable: ThrowableItem::default(),
            used: false,
        })
        .untyped()
    }

    pub fn throw(&mut self, force: bool) {
        self.throwable.throw(&mut self.body, force);
    }

    pub fn shoot(volcano: Handle<Volcano>, player: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            let mut volcano = scene::get_node(volcano);
            let player = &mut *scene::get_node(player);

            // `used` is still required, otherwise, spawning may be called multiple times.
            if volcano.used {
                player.state_machine.set_state(Player::ST_NORMAL);
                return;
            }

            volcano.used = true;

            EruptingVolcano::spawn(player.id);

            player.weapon = None;
            player.floating = false;

            volcano.delete();

            player.state_machine.set_state(Player::ST_NORMAL);
        };

        start_coroutine(coroutine)
    }

    pub fn weapon_capabilities() -> capabilities::Weapon {
        fn throw(node: HandleUntyped, force: bool) {
            let mut volcano = scene::get_untyped_node(node).unwrap().to_typed::<Volcano>();

            Volcano::throw(&mut *volcano, force)
        }

        fn shoot(node: HandleUntyped, player: Handle<Player>) -> Coroutine {
            let node = scene::get_untyped_node(node)
                .unwrap()
                .to_typed::<Volcano>()
                .handle();

            Volcano::shoot(node, player)
        }

        fn is_thrown(node: HandleUntyped) -> bool {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Volcano>();

            node.throwable.thrown()
        }

        fn pick_up(node: HandleUntyped, owner: Handle<Player>) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Volcano>();

            node.body.angle = 0.;
            node.throwable.owner = Some(owner);
        }

        fn mount(node: HandleUntyped, parent_pos: Vec2, parent_facing: bool) {
            let mut node = scene::get_untyped_node(node).unwrap().to_typed::<Volcano>();
            let mount_pos = if node.body.facing {
                vec2(5., 16.)
            } else {
                vec2(-40., 16.)
            };

            node.body.pos = parent_pos + mount_pos;
            node.body.facing = parent_facing;
        }

        fn collider(node: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(node).unwrap().to_typed::<Volcano>();
            Rect::new(
                node.body.pos.x,
                node.body.pos.y,
                Volcano::COLLIDER_WIDTH,
                Volcano::COLLIDER_HEIGHT,
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
                .to_typed::<Volcano>();

            node.throwable.owner.is_none()
        }
        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Volcano>();

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
                .to_typed::<Volcano>();
            node.body.velocity.x = speed;
        }
        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Volcano>();
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

impl Node for Volcano {
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
            .draw(node.body.pos, node.body.facing, node.body.angle);
    }
}

const SPAWNERS: [fn(Vec2, Vec2, f32, u8); 1] = [ArmedGrenade::spawn_for_volcano];

enum EruptingVolcanoState {
    Emerging,
    Erupting(f32),
    Submerging,
}

/// The EruptingVolcano doesn't have a body, as it doesn't have physics.
pub struct EruptingVolcano {
    sprite: AnimatedSprite,
    current_pos: Vec2,
    state: EruptingVolcanoState,
    last_shake_time: f32,
    time_to_throw_next_item: f32,
    owner_id: u8,
}

impl EruptingVolcano {
    const WIDTH: f32 = 395.;
    const HEIGHT: f32 = 100.;
    const ANIMATION_ERUPTING: &'static str = "erupting";
    /// Also applies to submersion
    const EMERSION_TIME: f32 = 5.1;
    const ERUPTION_TIME: f32 = 5.0;
    const APPROX_ERUPTED_ITEMS: u8 = 15;
    const SHAKE_INTERVAL: f32 = 0.2;

    /// Relative to the volcano top
    const MOUTH_Y: f32 = 30.;
    /// Relative to the volcano left
    const MOUTH_X_START: f32 = 118.;
    const MOUTH_X_LEN: f32 = 150.;

    /// Takes care of adding the FG to the node graph.
    pub fn spawn(owner_id: u8) {
        let erupting_volcano = Self::new(owner_id);

        scene::add_node(erupting_volcano);
    }

    fn new(owner_id: u8) -> Self {
        let erupting_volcano_sprite = AnimatedSprite::new(
            Self::WIDTH as u32,
            Self::HEIGHT as u32,
            &[Animation {
                name: Self::ANIMATION_ERUPTING.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            true,
        );

        let start_pos = Self::start_position();

        Self {
            sprite: erupting_volcano_sprite,
            current_pos: start_pos,
            state: EruptingVolcanoState::Emerging,
            last_shake_time: Self::SHAKE_INTERVAL,
            time_to_throw_next_item: Self::time_before_new_item(),
            owner_id,
        }
    }

    fn throw_item(&mut self) {
        let item_pos = Self::new_item_pos();
        let item_speed = Self::new_item_speed(item_pos.x);
        let item_enable_at_y = Self::new_item_enable_at_y();

        let spawner = SPAWNERS[gen_range(0, SPAWNERS.len())];

        spawner(item_pos, item_speed, item_enable_at_y, self.owner_id);
    }

    fn eruption_shake(mut erupting_volcano: RefMut<Self>) {
        if erupting_volcano.last_shake_time >= Self::SHAKE_INTERVAL {
            scene::find_node_by_type::<crate::nodes::Camera>().unwrap();
            erupting_volcano.last_shake_time = 0.;
        }
    }

    // POSITION HELPERS ////////////////////////////////////////////////////////

    /// Returns (start_posion, direction)
    fn start_position() -> Vec2 {
        let (map_width, map_height) = Self::map_dimensions();

        let start_x = (map_width - Self::WIDTH) / 2.;

        let start_y = map_height;

        vec2(start_x, start_y)
    }

    fn max_emersion_y() -> f32 {
        Self::map_dimensions().1 - Self::HEIGHT
    }

    /// Returns (map_width, map_height)
    fn map_dimensions() -> (f32, f32) {
        let world = storage::get::<GameWorld>();

        let map_size = world.map.get_size();

        (map_size.x, map_size.y)
    }

    // NEW ITEM HELPERS ////////////////////////////////////////////////////////

    fn time_before_new_item() -> f32 {
        // Multiply by two, so in average, we have the expected number of items.
        // Note that if an item is shot immediately, this logic will be more likely to shoot one item
        // more than intended.
        gen_range(
            0.,
            2. * Self::ERUPTION_TIME / Self::APPROX_ERUPTED_ITEMS as f32,
        )
    }

    fn new_item_speed(item_x: f32) -> Vec2 {
        let (map_width, map_height) = Self::map_dimensions();

        // Gravity law: vₜ² = vᵢ² - 2gy
        // We take the negative solution, since we go upwards.
        let y_speed = -(2. * PhysicsBody::GRAVITY * map_height).sqrt();

        let x_distance = if item_x >= map_width / 2. {
            map_width - item_x
        } else {
            -item_x
        };

        // Gravity law: t = (vₜ - vᵢ) / g
        // The 2* factor is due to going up, then down.
        let time_on_map: f32 = (-y_speed / PhysicsBody::GRAVITY) * 2.;
        let max_x_speed = x_distance / time_on_map;

        let x_speed = gen_range(0., max_x_speed);

        vec2(x_speed, y_speed)
    }

    fn new_item_pos() -> Vec2 {
        let item_y = Self::max_emersion_y() + Self::MOUTH_Y;

        let mouth_left_x = Self::start_position().x + Self::MOUTH_X_START;
        let item_x = gen_range(mouth_left_x, mouth_left_x + Self::MOUTH_X_LEN);

        vec2(item_x, item_y)
    }

    fn new_item_enable_at_y() -> f32 {
        let map_height = Self::map_dimensions().1;

        gen_range(0., map_height)
    }
}

impl Node for EruptingVolcano {
    fn fixed_update(mut erupting_volcano: RefMut<Self>) {
        match &mut erupting_volcano.state {
            EruptingVolcanoState::Emerging => {
                erupting_volcano.current_pos.y -=
                    (Self::HEIGHT / Self::EMERSION_TIME) * get_frame_time();

                if erupting_volcano.current_pos.y <= Self::max_emersion_y() {
                    erupting_volcano.state = EruptingVolcanoState::Erupting(0.);
                } else {
                    erupting_volcano.last_shake_time += get_frame_time();

                    Self::eruption_shake(erupting_volcano);
                }
            }
            EruptingVolcanoState::Erupting(time) => {
                *time += get_frame_time();

                if *time >= Self::ERUPTION_TIME {
                    erupting_volcano.state = EruptingVolcanoState::Submerging;
                    erupting_volcano.last_shake_time = Self::SHAKE_INTERVAL;
                } else {
                    erupting_volcano.time_to_throw_next_item -= get_frame_time();

                    if erupting_volcano.time_to_throw_next_item <= 0. {
                        erupting_volcano.throw_item();
                        erupting_volcano.time_to_throw_next_item = Self::time_before_new_item();
                    }
                }
            }
            EruptingVolcanoState::Submerging => {
                erupting_volcano.current_pos.y +=
                    (Self::HEIGHT / Self::EMERSION_TIME) * get_frame_time();

                if erupting_volcano.current_pos.y >= Self::map_dimensions().1 {
                    erupting_volcano.delete();
                } else {
                    erupting_volcano.last_shake_time += get_frame_time();

                    Self::eruption_shake(erupting_volcano);
                }
            }
        }
    }

    fn draw(mut erupting_volcano: RefMut<Self>) {
        erupting_volcano.sprite.update();

        let resources = storage::get::<Resources>();
        let texture_entry = resources.textures.get("volcano").unwrap();

        draw_texture_ex(
            texture_entry.texture,
            erupting_volcano.current_pos.x,
            erupting_volcano.current_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(erupting_volcano.sprite.frame().source_rect),
                dest_size: Some(erupting_volcano.sprite.frame().dest_size),
                flip_x: false,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}
