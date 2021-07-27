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

use super::player::{PLAYER_HITBOX_HEIGHT, PLAYER_HITBOX_WIDTH};

const RAINING_SHARK_WIDTH: f32 = 60.;
const RAINING_SHARK_HEIGHT: f32 = 220.;
const RAINING_SHARK_ANIMATION_RAINING: &'static str = "raining";

const SHARKS_COUNT: usize = 5;
const RAINING_SHARK_SPEED: f32 = 350.;
/// Determines the vertical spread of the sharks
const MIN_START_Y: f32 = -400. - RAINING_SHARK_HEIGHT;

/// RainingShark's don't have a body, as they don't have physics.
pub struct RainingShark {
    sprite: AnimatedSprite,
    current_pos: Vec2,
    direction: bool,
    owner_id: u8,
}

// No new() - RainingShark's are essentially added as nodes, and forgotten.
impl RainingShark {
    pub fn rain(owner_id: u8) {
        let sprite = AnimatedSprite::new(
            RAINING_SHARK_WIDTH as u32,
            RAINING_SHARK_HEIGHT as u32,
            &[Animation {
                name: RAINING_SHARK_ANIMATION_RAINING.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            true,
        );

        let raining_shark_positions = Self::compute_raining_shark_positions();
        let raining_sharks = raining_shark_positions
            .iter()
            .map(|pos| RainingShark {
                sprite: sprite.clone(),
                current_pos: *pos,
                direction: gen_range(0., 1.) < 0.5,
                owner_id,
            })
            .collect::<Vec<_>>();

        for raining_shark in raining_sharks {
            scene::add_node(raining_shark);
        }
    }

    // Uses the st00pidest possible algorithm for not superposing sharks.
    //
    // The strict and not st00pid algorithm is likely a series of random weighted choices, using a list
    // of available slots, and placing each shark via binary search (tot.: n log₂n).
    //
    fn compute_raining_shark_positions() -> Vec<Vec2> {
        let mut x_positions: Vec<u32> = vec![];
        let map_width = {
            let resources = storage::get::<Resources>();
            resources.tiled_map.raw_tiled_map.tilewidth * resources.tiled_map.raw_tiled_map.width
        };

        while x_positions.len() < SHARKS_COUNT {
            let attempted_x = gen_range(0, map_width);

            let position_ok = x_positions.iter().all(|existing_x| {
                attempted_x >= existing_x + RAINING_SHARK_WIDTH as u32
                    || attempted_x <= existing_x.saturating_sub(RAINING_SHARK_WIDTH as u32)
            });

            if position_ok {
                x_positions.push(attempted_x);
            }
        }

        x_positions
            .iter()
            .map(|x| vec2(*x as f32, gen_range(MIN_START_Y, -RAINING_SHARK_HEIGHT)))
            .collect()
    }
}

impl scene::Node for RainingShark {
    fn fixed_update(mut shark: RefMut<Self>) {
        let map_height = {
            let resources = storage::get::<Resources>();
            (resources.tiled_map.raw_tiled_map.tileheight
                * resources.tiled_map.raw_tiled_map.height) as f32
        };

        shark.current_pos.y += get_frame_time() * RAINING_SHARK_SPEED;

        if shark.current_pos.y >= map_height {
            shark.delete();
            return;
        }

        let shark_hitbox = Rect::new(
            shark.current_pos.x,
            shark.current_pos.y,
            RAINING_SHARK_WIDTH,
            RAINING_SHARK_HEIGHT,
        );

        for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
            // The shark is large, so we need to avoid triggering this routine multiple times.
            if player.dead {
                continue;
            }

            if shark.owner_id != player.id {
                let player_hitbox = Rect::new(
                    player.body.pos.x,
                    player.body.pos.y,
                    PLAYER_HITBOX_WIDTH,
                    PLAYER_HITBOX_HEIGHT,
                );

                if player_hitbox.intersect(shark_hitbox).is_some() {
                    scene::find_node_by_type::<crate::nodes::Camera>()
                        .unwrap()
                        .shake();

                    let direction =
                        shark.current_pos.x > (player.body.pos.x + PLAYER_HITBOX_WIDTH / 2.);
                    player.kill(direction);
                }
            }
        }
    }

    fn draw(mut shark: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();
        shark.sprite.update();

        draw_texture_ex(
            resources.raining_shark,
            shark.current_pos.x,
            shark.current_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(shark.sprite.frame().source_rect),
                dest_size: Some(shark.sprite.frame().dest_size),
                flip_x: shark.direction,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}
