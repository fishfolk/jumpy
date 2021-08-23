use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::RefMut,
    },
    prelude::*,
    rand::gen_range,
};

use crate::{nodes::ArmedGrenade, Resources};

use super::{player::PhysicsBody, Cannonball, EruptedItem};

const VOLCANO_WIDTH: f32 = 395.;
const VOLCANO_HEIGHT: f32 = 100.;
const VOLCANO_ANIMATION_ERUPTING: &str = "erupting";
/// Also applies to submersion
const VOLCANO_EMERSION_TIME: f32 = 5.1;
const VOLCANO_ERUPTION_TIME: f32 = 5.0;
const VOLCANO_APPROX_ERUPTED_ITEMS: u8 = 15;
const SHAKE_INTERVAL: f32 = 0.2;

/// Relative to the volcano top
const VOLCANO_MOUTH_Y: f32 = 30.;
/// Relative to the volcano left
const VOLCANO_MOUTH_X_START: f32 = 118.;
const VOLCANO_MOUTH_X_LEN: f32 = 150.;

const SPAWNERS: [fn(Vec2, Vec2, f32, u8); 2] = [
    ArmedGrenade::spawn_for_volcano,
    Cannonball::spawn_for_volcano,
];

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
    /// Takes care of adding the FG to the node graph.
    pub fn spawn(owner_id: u8) {
        let erupting_volcano = Self::new(owner_id);

        scene::add_node(erupting_volcano);
    }

    fn new(owner_id: u8) -> Self {
        let erupting_volcano_sprite = AnimatedSprite::new(
            VOLCANO_WIDTH as u32,
            VOLCANO_HEIGHT as u32,
            &[Animation {
                name: VOLCANO_ANIMATION_ERUPTING.to_string(),
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
            last_shake_time: SHAKE_INTERVAL,
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
        if erupting_volcano.last_shake_time >= SHAKE_INTERVAL {
            scene::find_node_by_type::<crate::nodes::Camera>()
                .unwrap()
                .shake_noise(0.5, 18, 0.5);
            erupting_volcano.last_shake_time = 0.;
        }
    }

    // POSITION HELPERS ////////////////////////////////////////////////////////

    /// Returns (start_posion, direction)
    fn start_position() -> Vec2 {
        let (map_width, map_height) = Self::map_dimensions();

        let start_x = (map_width - VOLCANO_WIDTH) / 2.;

        let start_y = map_height;

        vec2(start_x, start_y)
    }

    fn max_emersion_y() -> f32 {
        Self::map_dimensions().1 - VOLCANO_HEIGHT
    }

    /// Returns (map_width, map_height)
    fn map_dimensions() -> (f32, f32) {
        let resources = storage::get::<Resources>();

        let map_width = (resources.tiled_map.raw_tiled_map.tilewidth
            * resources.tiled_map.raw_tiled_map.width) as f32;
        let map_height = (resources.tiled_map.raw_tiled_map.tileheight
            * resources.tiled_map.raw_tiled_map.height) as f32;

        (map_width, map_height)
    }

    // NEW ITEM HELPERS ////////////////////////////////////////////////////////

    fn time_before_new_item() -> f32 {
        // Multiply by two, so in average, we have the expected number of items.
        // Note that if an item is shot immediately, this logic will be more likely to shoot one item
        // more than intended.
        gen_range(
            0.,
            2. * VOLCANO_ERUPTION_TIME / VOLCANO_APPROX_ERUPTED_ITEMS as f32,
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
        let item_y = Self::max_emersion_y() + VOLCANO_MOUTH_Y;

        let mouth_left_x = Self::start_position().x + VOLCANO_MOUTH_X_START;
        let item_x = gen_range(mouth_left_x, mouth_left_x + VOLCANO_MOUTH_X_LEN);

        vec2(item_x, item_y)
    }

    fn new_item_enable_at_y() -> f32 {
        let map_height = Self::map_dimensions().1;

        gen_range(0., map_height)
    }
}

impl scene::Node for EruptingVolcano {
    fn fixed_update(mut erupting_volcano: RefMut<Self>) {
        match &mut erupting_volcano.state {
            EruptingVolcanoState::Emerging => {
                erupting_volcano.current_pos.y -=
                    (VOLCANO_HEIGHT / VOLCANO_EMERSION_TIME) * get_frame_time();

                if erupting_volcano.current_pos.y <= Self::max_emersion_y() {
                    erupting_volcano.state = EruptingVolcanoState::Erupting(0.);
                } else {
                    erupting_volcano.last_shake_time += get_frame_time();

                    Self::eruption_shake(erupting_volcano);
                }
            }
            EruptingVolcanoState::Erupting(time) => {
                *time += get_frame_time();

                if *time >= VOLCANO_ERUPTION_TIME {
                    erupting_volcano.state = EruptingVolcanoState::Submerging;
                    erupting_volcano.last_shake_time = SHAKE_INTERVAL;
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
                    (VOLCANO_HEIGHT / VOLCANO_EMERSION_TIME) * get_frame_time();

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
        let resources = storage::get_mut::<Resources>();
        erupting_volcano.sprite.update();

        draw_texture_ex(
            resources.erupting_volcano,
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
