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

use crate::Resources;

use super::cannonball::Cannonball;

const VOLCANO_WIDTH: f32 = 395.;
const VOLCANO_HEIGHT: f32 = 100.;
const VOLCANO_ANIMATION_ERUPTING: &'static str = "erupting";
/// Also applies to submersion
const VOLCANO_EMERSION_TIME: f32 = 3.0;
const VOLCANO_ERUPTION_TIME: f32 = 5.0;
const VOLCANO_APPROX_ERUPTED_ITEMS: u8 = 32;

/// Relative to the volcano top
const VOLCANO_MOUTH_Y: f32 = 30.;
/// Relative to the volcano left
const VOLCANO_MOUTH_X_START: f32 = 118.;
const VOLCANO_MOUTH_X_LEN: f32 = 150.;

// Based on these two, the max (initial) speed is calculated.
// For simplicity, on the way up, the standard gravity law applies, but on the way back, there is a
// cap on speed.
const GRAVITY: f32 = 700.;
/// Time to reach the top of the map
const ITEMS_TIME_TO_CROSS_MAP: f32 = 4.;
const MAX_FALLING_SPEED: f32 = 500.;

const SHAKE_GRACE_TIME: f32 = 0.2;

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
    thrown_items: Vec<Cannonball>,
}

impl EruptingVolcano {
    /// Takes care of adding the FG to the node graph.
    pub fn spawn() {
        let erupting_volcano = Self::new();

        scene::add_node(erupting_volcano);
    }

    fn new() -> Self {
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
            last_shake_time: SHAKE_GRACE_TIME,
            time_to_throw_next_item: Self::time_for_next_item(),
            thrown_items: vec![],
        }
    }

    fn throw_item() {
        let item_speed = Self::start_items_max_speed();
        let item_pos = Self::pos_for_next_item();

        println!("WRITEME: throwing item: {}, {}", item_pos, item_speed);
    }

    /// Returns (start_posion, direction)
    fn start_position() -> Vec2 {
        let (map_width, map_height) = Self::map_dimensions();

        let start_x = (map_width - VOLCANO_WIDTH) / 2.;

        let start_y = map_height;

        vec2(start_x, start_y)
    }

    fn start_items_max_speed() -> f32 {
        let (_, map_height) = Self::map_dimensions();

        // Gravity law: y = gt²/2 + vᵢt
        (map_height - GRAVITY * VOLCANO_EMERSION_TIME.powi(2) / 2.) / VOLCANO_EMERSION_TIME
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

    fn max_emersion_y() -> f32 {
        Self::map_dimensions().1 - VOLCANO_HEIGHT
    }

    fn eruption_shake(mut erupting_volcano: RefMut<Self>) {
        if erupting_volcano.last_shake_time >= SHAKE_GRACE_TIME {
            scene::find_node_by_type::<crate::nodes::Camera>()
                .unwrap()
                .shake();
            erupting_volcano.last_shake_time = 0.;
        }
    }

    fn time_for_next_item() -> f32 {
        // Multiply by two, so in average, we have the expected number of items.
        // Note that if an item is shot immediately, this logic will be more likely to shoot one item
        // more than intended.
        gen_range(
            0.,
            2. * VOLCANO_ERUPTION_TIME / VOLCANO_APPROX_ERUPTED_ITEMS as f32,
        )
    }

    fn pos_for_next_item() -> Vec2 {
        let item_y = Self::max_emersion_y() + VOLCANO_MOUTH_Y;

        let mouth_left_x = Self::start_position().x + VOLCANO_MOUTH_X_START;
        let item_x = gen_range(mouth_left_x, mouth_left_x + VOLCANO_MOUTH_X_LEN);

        vec2(item_x, item_y)
    }
}

impl scene::Node for EruptingVolcano {
    fn fixed_update(mut erupting_volcano: RefMut<Self>) {
        println!("WRITEME: Complete EruptingVolcano fixed_update()");

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
                    erupting_volcano.last_shake_time = SHAKE_GRACE_TIME;
                } else {
                    erupting_volcano.time_to_throw_next_item -= get_frame_time();

                    if erupting_volcano.time_to_throw_next_item <= 0. {
                        Self::throw_item();
                        erupting_volcano.time_to_throw_next_item = Self::time_for_next_item();
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

        //         let direction_x_factor = if erupting_volcano.direction { 1. } else { -1. };
        //         let map_width = {
        //             let resources = storage::get::<Resources>();
        //             (resources.tiled_map.raw_tiled_map.tilewidth * resources.tiled_map.raw_tiled_map.width)
        //                 as f32
        //         };
        //
        //         erupting_volcano.current_pos.x +=
        //             direction_x_factor * get_frame_time() * VOLCANO_SPEED;
        //
        //         if erupting_volcano.current_pos.x + VOLCANO_WIDTH < 0.
        //             || erupting_volcano.current_pos.x > map_width - 1.
        //         {
        //             erupting_volcano.delete();
        //             return;
        //         }
        //
        //         let erupting_volcano_hitbox = Rect::new(
        //             erupting_volcano.current_pos.x,
        //             erupting_volcano.current_pos.y,
        //             VOLCANO_WIDTH,
        //             VOLCANO_HEIGHT,
        //         );
        //
        //         for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
        //             // The vessel is very large, so we need to avoid triggering this routine multiple times.
        //             if player.dead {
        //                 continue;
        //             }
        //
        //             if erupting_volcano.owner_id != player.id {
        //                 let player_hitbox = Rect::new(
        //                     player.body.pos.x,
        //                     player.body.pos.y,
        //                     PLAYER_HITBOX_WIDTH,
        //                     PLAYER_HITBOX_HEIGHT,
        //                 );
        //
        //                 if player_hitbox.intersect(erupting_volcano_hitbox).is_some() {
        //                     scene::find_node_by_type::<crate::nodes::Camera>()
        //                         .unwrap()
        //                         .shake();
        //
        //                     let direction = erupting_volcano.current_pos.x
        //                         > (player.body.pos.x + PLAYER_HITBOX_WIDTH / 2.);
        //                     player.kill(direction);
        //                 }
        //             }
        //         }
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
