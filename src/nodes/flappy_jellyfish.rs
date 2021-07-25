use macroquad::{
    color,
    experimental::animation::{AnimatedSprite, Animation},
    prelude::{collections::storage, scene::RefMut, *},
};
use macroquad_platformer::Tile;

use crate::Resources;

use super::{
    jellyfish::MountStatus,
    player::{PLAYER_HITBOX_HEIGHT, PLAYER_HITBOX_WIDTH},
    Jellyfish, Player,
};

const FLAPPY_JELLYFISH_WIDTH: f32 = 50.;
pub const FLAPPY_JELLYFISH_HEIGHT: f32 = 51.;
const FLAPPY_JELLYFISH_ANIMATION_FLAPPY: &'static str = "flappy";
const FLAPPY_JELLYFISH_X_SPEED: f32 = 200.;
// Use the player width, for simplicity
const FLAPPY_JELLYFISH_SPAWN_X_DISTANCE: f32 = 76.;
const JUMP_SPEED: f32 = -500.;
const GRAVITY: f32 = 700.;
const ABSOLUTE_MAX_SPEED: f32 = 300.;

/// The FlappyJellyfish doesn't have a body, as it has a non-conventional (flappy bird-style) motion.
pub struct FlappyJellyfish {
    flappy_jellyfish_sprite: AnimatedSprite,
    current_pos: Vec2,
    /// Positive: downwards
    current_y_speed: f32,
    owner_id: u8,
}

/// This type is dynamically added and removed from the scene graph, as it's the simplest way.
impl FlappyJellyfish {
    /// Not to be called; call spawn() instead, which handles the node and position.
    fn new(jellyfish_pos: Vec2, owner_id: u8) -> Self {
        let flappy_jellyfish_sprite = AnimatedSprite::new(
            FLAPPY_JELLYFISH_WIDTH as u32,
            FLAPPY_JELLYFISH_HEIGHT as u32,
            &[Animation {
                name: FLAPPY_JELLYFISH_ANIMATION_FLAPPY.to_string(),
                row: 0,
                frames: 8,
                fps: 8,
            }],
            true,
        );

        Self {
            flappy_jellyfish_sprite,
            current_pos: jellyfish_pos,
            current_y_speed: JUMP_SPEED,
            owner_id,
        }
    }

    /// Returns true if the jellyfish was successfully spawned.
    /// It won't spawn if colliding a solid.
    pub fn spawn(jellyfish: &mut Jellyfish, owner: &mut Player) -> bool {
        let direction_x_factor = if jellyfish.body.facing { 1. } else { -1. };
        let flappy_jellyfish_pos =
            jellyfish.body.pos + vec2(direction_x_factor * FLAPPY_JELLYFISH_SPAWN_X_DISTANCE, 0.);

        let collides_solid = {
            let resources = storage::get_mut::<Resources>();

            resources.collision_world.collide_solids(
                flappy_jellyfish_pos,
                FLAPPY_JELLYFISH_WIDTH as i32,
                FLAPPY_JELLYFISH_HEIGHT as i32,
            ) == Tile::Solid
        };

        if !collides_solid {
            let flappy_jellyfish = Self::new(flappy_jellyfish_pos, owner.id);

            scene::add_node(flappy_jellyfish);

            jellyfish.mount_status = MountStatus::Driving;

            owner.remote_control = true;
        }

        !collides_solid
    }

    /// Handles everything, but needs access to the player/jellyfish nodes, so they must not be in scope.
    pub fn terminate(flappy_jellyfish: RefMut<FlappyJellyfish>, killed_player_ids: Vec<u8>) {
        let hit_fxses = &mut storage::get_mut::<Resources>().hit_fxses;
        let explosion_position = vec2(
            flappy_jellyfish.current_pos.x + FLAPPY_JELLYFISH_WIDTH / 2.,
            flappy_jellyfish.current_pos.y + FLAPPY_JELLYFISH_HEIGHT / 2.,
        );
        hit_fxses.spawn(explosion_position);

        let mut jellyfish = scene::find_node_by_type::<Jellyfish>().unwrap();
        jellyfish.mount_status = MountStatus::Dismounted;

        let mut owner = scene::find_nodes_by_type::<Player>()
            .find(|player| player.id == flappy_jellyfish.owner_id)
            .unwrap();
        owner.remote_control = false;
        // If the killed player is the owner, don't set the state, otherwise, it will override the death
        // state!
        if !killed_player_ids.contains(&owner.id) {
            owner.state_machine.set_state(Player::ST_NORMAL);
        }

        flappy_jellyfish.delete();
    }
}

impl scene::Node for FlappyJellyfish {
    fn fixed_update(mut flappy_jellyfish: RefMut<Self>) {
        // The termination logic needs to be always separate, due to entities scoping or BCK.

        let terminate_flappy_jellyfish = {
            let player = scene::find_nodes_by_type::<crate::nodes::Player>()
                .find(|p| p.id == flappy_jellyfish.owner_id)
                .unwrap();

            if player.input.jump {
                flappy_jellyfish.current_y_speed += JUMP_SPEED;
            }
            if player.input.left {
                flappy_jellyfish.current_pos +=
                    vec2(-FLAPPY_JELLYFISH_X_SPEED * get_frame_time(), 0.);
            }
            if player.input.right {
                flappy_jellyfish.current_pos +=
                    vec2(FLAPPY_JELLYFISH_X_SPEED * get_frame_time(), 0.);
            }

            // It's crucial to inspect tapping here, not pressing, otherwise, the shoot() keypress will
            // flow here, causing immediate termination on spawn!
            player.input.fire
        };

        if terminate_flappy_jellyfish {
            FlappyJellyfish::terminate(flappy_jellyfish, vec![]);
            return;
        }

        // Displacement formula: `y = gt²/2 + vᵢt`
        // Speed formula: `vₜ = vᵢ + tg`
        //
        flappy_jellyfish.current_y_speed = (flappy_jellyfish.current_y_speed
            + get_frame_time() * GRAVITY)
            .clamp(-ABSOLUTE_MAX_SPEED, ABSOLUTE_MAX_SPEED);

        let fall_displacement = GRAVITY * get_frame_time().powi(2) / 2.
            + flappy_jellyfish.current_y_speed * get_frame_time();
        flappy_jellyfish.current_pos += vec2(0., fall_displacement);

        // Check/act on map borders

        let (map_width, map_height) = {
            let resources = storage::get::<Resources>();

            let width = resources.tiled_map.raw_tiled_map.tilewidth
                * resources.tiled_map.raw_tiled_map.width;
            let height = resources.tiled_map.raw_tiled_map.tileheight
                * resources.tiled_map.raw_tiled_map.height;

            (width as f32, height as f32)
        };

        if flappy_jellyfish.current_pos.x < 0.
            || flappy_jellyfish.current_pos.x > map_width as f32
            || flappy_jellyfish.current_pos.y < 0.
            || flappy_jellyfish.current_pos.y > map_height as f32
        {
            FlappyJellyfish::terminate(flappy_jellyfish, vec![]);
            return;
        }

        // Check/act on player collisions

        let flappy_jellyfish_hitbox = Rect::new(
            flappy_jellyfish.current_pos.x,
            flappy_jellyfish.current_pos.y,
            FLAPPY_JELLYFISH_WIDTH,
            FLAPPY_JELLYFISH_HEIGHT,
        );

        let killed_player_ids = scene::find_nodes_by_type::<crate::nodes::Player>().fold(
            vec![],
            |mut killed_player_ids, mut player| {
                let player_hitbox = Rect::new(
                    player.body.pos.x,
                    player.body.pos.y,
                    PLAYER_HITBOX_WIDTH,
                    PLAYER_HITBOX_HEIGHT,
                );
                if player_hitbox.intersect(flappy_jellyfish_hitbox).is_some() {
                    scene::find_node_by_type::<crate::nodes::Camera>()
                        .unwrap()
                        .shake();

                    let direction = flappy_jellyfish.current_pos.x
                        > (player.body.pos.x + PLAYER_HITBOX_WIDTH / 2.);
                    player.kill(direction);

                    killed_player_ids.push(player.id);
                }

                killed_player_ids
            },
        );

        if killed_player_ids.len() > 0 {
            FlappyJellyfish::terminate(flappy_jellyfish, killed_player_ids);
        }
    }

    fn draw(mut flappy_jellyfish: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        flappy_jellyfish.flappy_jellyfish_sprite.update();

        draw_texture_ex(
            resources.flappy_jellyfishes,
            flappy_jellyfish.current_pos.x,
            flappy_jellyfish.current_pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(flappy_jellyfish.flappy_jellyfish_sprite.frame().source_rect),
                dest_size: Some(flappy_jellyfish.flappy_jellyfish_sprite.frame().dest_size),
                flip_x: false,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}
