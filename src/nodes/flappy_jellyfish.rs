use macroquad::{
    color,
    experimental::animation::{AnimatedSprite, Animation},
    prelude::{collections::storage, scene::RefMut, *},
};
use macroquad_platformer::Tile;

use crate::{items::jellyfish::Jellyfish, items::jellyfish::MountStatus, nodes::Player, Resources};

/// The FlappyJellyfish doesn't have a body, as it has a non-conventional (flappy bird-style) motion.
pub struct FlappyJellyfish {
    flappy_jellyfish_sprite: AnimatedSprite,
    current_pos: Vec2,
    /// Positive: downwards
    current_y_speed: f32,
    owner_id: u8,
    previous_fire_state: bool,
    can_jump: bool,
}

/// This type is dynamically added and removed from the scene graph, as it's the simplest way.
impl FlappyJellyfish {
    const COLLIDER_WIDTH: f32 = 34.;
    pub const COLLIDER_HEIGHT: f32 = 47.;
    const X_SPEED: f32 = 200.;
    // Use the player width, for simplicity
    const SPAWN_X_DISTANCE: f32 = 76.;
    const JUMP_SPEED: f32 = -500.;
    const GRAVITY: f32 = 700.;
    const ABSOLUTE_MAX_SPEED: f32 = 300.;

    /// Not to be called; call spawn() instead, which handles the node and position.
    fn new(jellyfish_pos: Vec2, owner_id: u8) -> Self {
        let flappy_jellyfish_sprite = AnimatedSprite::new(
            Self::COLLIDER_WIDTH as u32,
            Self::COLLIDER_HEIGHT as u32,
            &[Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 3,
                fps: 8,
            }],
            true,
        );

        Self {
            flappy_jellyfish_sprite,
            current_pos: jellyfish_pos,
            current_y_speed: Self::JUMP_SPEED,
            owner_id,
            previous_fire_state: true,
            can_jump: true,
        }
    }

    /// Returns true if the jellyfish was successfully spawned.
    /// It won't spawn if colliding a solid.
    pub fn spawn(jellyfish: &mut Jellyfish, owner: &mut Player) -> bool {
        let direction_x_factor = if jellyfish.body.facing { 1. } else { -1. };
        let flappy_jellyfish_pos =
            jellyfish.body.pos + vec2(direction_x_factor * Self::SPAWN_X_DISTANCE, 0.);

        let collides_solid = {
            let resources = storage::get_mut::<Resources>();

            resources.collision_world.collide_solids(
                flappy_jellyfish_pos,
                Self::COLLIDER_WIDTH as i32,
                Self::COLLIDER_HEIGHT as i32,
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
            flappy_jellyfish.current_pos.x + Self::COLLIDER_WIDTH / 2.,
            flappy_jellyfish.current_pos.y + Self::COLLIDER_HEIGHT / 2.,
        );
        hit_fxses.spawn(explosion_position);

        let mut jellyfish = scene::find_node_by_type::<Jellyfish>().unwrap();
        if !matches!(jellyfish.mount_status, MountStatus::Dropped) {
            jellyfish.mount_status = MountStatus::Dismounted;
        }

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
        // It's crucial to inspect tapping, not pressing, otherwise, the shoot() keypress will flow
        // here, causing immediate termination on spawn!
        // For this reason, on spawning, previous_fire_state must be set to true.
        let fire_tapped = {
            let player = scene::find_nodes_by_type::<crate::nodes::Player>()
                .find(|p| p.id == flappy_jellyfish.owner_id)
                .unwrap();

            let fire_tapped = player.input.fire && !flappy_jellyfish.previous_fire_state;
            flappy_jellyfish.previous_fire_state = player.input.fire;
            fire_tapped
        };

        // Termination

        let jellyfish_mount_status = {
            scene::find_node_by_type::<Jellyfish>()
                .unwrap()
                .mount_status
        };

        // MountStatus::Dropped check is a fix for case when fish killed while mounting flappy jellyfish
        if fire_tapped || matches!(jellyfish_mount_status, MountStatus::Dropped) {
            FlappyJellyfish::terminate(flappy_jellyfish, vec![]);
            return;
        }

        // Movement
        //
        // Displacement formula: `y = gt²/2 + vᵢt`
        // Speed formula: `vₜ = vᵢ + tg`

        let mut diff_pos = vec2(0., 0.);
        let mut diff_y_speed = 0.;

        {
            let player = scene::find_nodes_by_type::<crate::nodes::Player>()
                .find(|p| p.id == flappy_jellyfish.owner_id)
                .unwrap();

            if player.input.jump {
                //
                if flappy_jellyfish.can_jump {
                    flappy_jellyfish.can_jump = false;
                    diff_y_speed += Self::JUMP_SPEED;
                }
            } else {
                flappy_jellyfish.can_jump = true
            }
            if player.input.left {
                diff_pos += vec2(-Self::X_SPEED * get_frame_time(), 0.);
            }
            if player.input.right {
                diff_pos += vec2(Self::X_SPEED * get_frame_time(), 0.);
            }

            let y_speed_before_gravity = flappy_jellyfish.current_y_speed + diff_y_speed;
            flappy_jellyfish.current_y_speed = (y_speed_before_gravity
                + get_frame_time() * Self::GRAVITY)
                .clamp(-Self::ABSOLUTE_MAX_SPEED, Self::ABSOLUTE_MAX_SPEED);

            let fall_displacement = Self::GRAVITY * get_frame_time().powi(2) / 2.
                + flappy_jellyfish.current_y_speed * get_frame_time();
            diff_pos += vec2(0., fall_displacement);
        }

        let new_pos = flappy_jellyfish.current_pos + diff_pos;

        let collides_solid = {
            let resources = storage::get_mut::<Resources>();

            resources.collision_world.collide_solids(
                new_pos,
                Self::COLLIDER_WIDTH as i32,
                Self::COLLIDER_HEIGHT as i32,
            ) == Tile::Solid
        };

        if !collides_solid {
            flappy_jellyfish.current_pos = new_pos;

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
        }

        // Check/act on player collisions

        let flappy_jellyfish_hitbox = Rect::new(
            flappy_jellyfish.current_pos.x,
            flappy_jellyfish.current_pos.y,
            Self::COLLIDER_WIDTH,
            Self::COLLIDER_HEIGHT,
        );

        let killed_player_ids = scene::find_nodes_by_type::<crate::nodes::Player>().fold(
            vec![],
            |mut killed_player_ids, mut player| {
                let player_hitbox = player.get_hitbox();

                if player_hitbox.intersect(flappy_jellyfish_hitbox).is_some() {
                    scene::find_node_by_type::<crate::nodes::Camera>().unwrap();

                    let direction =
                        flappy_jellyfish.current_pos.x > (player.body.pos.x + player_hitbox.w / 2.);
                    player.kill(direction);

                    killed_player_ids.push(player.id);
                }

                killed_player_ids
            },
        );

        if !killed_player_ids.is_empty() {
            FlappyJellyfish::terminate(flappy_jellyfish, killed_player_ids);
        }
    }

    fn draw(mut flappy_jellyfish: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        flappy_jellyfish.flappy_jellyfish_sprite.update();

        draw_texture_ex(
            resources.flappy_jellyfish,
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
