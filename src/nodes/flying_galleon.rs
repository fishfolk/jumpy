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

const FLYING_GALLEON_WIDTH: f32 = 326.;
const FLYING_GALLEON_HEIGHT: f32 = 300.;
const FLYING_GALLEON_ANIMATION_FLYING: &str = "flying";
const FLYING_GALLEON_SPEED: f32 = 150.;

/// The FlyingGalleon doesn't have a body, as it doesn't have physics.
pub struct FlyingGalleon {
    flying_galleon_sprite: AnimatedSprite,
    current_pos: Vec2,
    direction: bool,
    owner_id: u8,
}

impl FlyingGalleon {
    /// Takes care of adding the FG to the node graph.
    pub fn spawn(owner_id: u8) {
        let flying_galleon = Self::new(owner_id);

        scene::add_node(flying_galleon);
    }

    fn new(owner_id: u8) -> Self {
        let flying_galleon_sprite = AnimatedSprite::new(
            FLYING_GALLEON_WIDTH as u32,
            FLYING_GALLEON_HEIGHT as u32,
            &[Animation {
                name: FLYING_GALLEON_ANIMATION_FLYING.to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            true,
        );

        let (start_pos, direction) = Self::start_position_data();

        Self {
            flying_galleon_sprite,
            current_pos: start_pos,
            direction,
            owner_id,
        }
    }

    /// Returns (start_posion, direction)
    fn start_position_data() -> (Vec2, bool) {
        let resources = storage::get::<Resources>();

        let map_width =
            resources.tiled_map.raw_tiled_map.tilewidth * resources.tiled_map.raw_tiled_map.width;
        let map_height =
            resources.tiled_map.raw_tiled_map.tileheight * resources.tiled_map.raw_tiled_map.height;

        let (start_x, direction) = if gen_range(0., 1.) < 0.5 {
            (0. - FLYING_GALLEON_WIDTH, true)
        } else {
            ((map_width - 1) as f32, false)
        };

        let start_y = gen_range(0., map_height as f32 - FLYING_GALLEON_HEIGHT);

        (vec2(start_x, start_y), direction)
    }
}

impl scene::Node for FlyingGalleon {
    fn fixed_update(mut flying_galleon: RefMut<Self>) {
        let direction_x_factor = if flying_galleon.direction { 1. } else { -1. };
        let map_width = {
            let resources = storage::get::<Resources>();
            (resources.tiled_map.raw_tiled_map.tilewidth * resources.tiled_map.raw_tiled_map.width)
                as f32
        };

        flying_galleon.current_pos.x +=
            direction_x_factor * get_frame_time() * FLYING_GALLEON_SPEED;

        if flying_galleon.current_pos.x + FLYING_GALLEON_WIDTH < 0.
            || flying_galleon.current_pos.x > map_width - 1.
        {
            flying_galleon.delete();
            return;
        }

        let flying_galleon_hitbox = Rect::new(
            flying_galleon.current_pos.x,
            flying_galleon.current_pos.y,
            FLYING_GALLEON_WIDTH,
            FLYING_GALLEON_HEIGHT,
        );

        for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
            // The vessel is very large, so we need to avoid triggering this routine multiple times.
            if player.dead {
                continue;
            }

            if flying_galleon.owner_id != player.id {
                let player_hitbox = player.get_hitbox();

                if player_hitbox.intersect(flying_galleon_hitbox).is_some() {
                    scene::find_node_by_type::<crate::nodes::Camera>()
                        .unwrap()
                        .shake();

                    let direction =
                        flying_galleon.current_pos.x > (player.body.pos.x + player_hitbox.w / 2.);
                    player.kill(direction);
                }
            }
        }
    }

    fn draw(mut flying_galleon: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();
        flying_galleon.flying_galleon_sprite.update();

        draw_texture_ex(
            resources.flying_galleon,
            flying_galleon.current_pos.x,
            flying_galleon.current_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(flying_galleon.flying_galleon_sprite.frame().source_rect),
                dest_size: Some(flying_galleon.flying_galleon_sprite.frame().dest_size),
                flip_x: flying_galleon.direction,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}
