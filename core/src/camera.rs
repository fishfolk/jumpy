use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    // Spawn the camera and parallax background.
    session
        .world
        .run_system(
            |mut entities: ResMut<Entities>,
             mut cameras: CompMut<Camera>,
             mut transforms: CompMut<Transform>,
             mut camera_shakes: CompMut<CameraShake>,
             mut camera_follows: CompMut<CameraState>| {
                let ent = entities.create();

                camera_shakes.insert(ent, CameraShake::new(6.0, glam::vec2(3.0, 3.0), 1.0));
                cameras.insert(ent, default());
                transforms.insert(ent, default());
                camera_follows.insert(ent, default());
            },
        )
        .unwrap();

    session
        .stages
        .add_system_to_stage(CoreStage::Last, camera_controller);
    session
        .stages
        .add_system_to_stage(CoreStage::Last, camera_parallax);
}

#[derive(Clone, TypeUlid)]
#[ulid = "01GPP1V3PCENFWC8H6H705ST80"]
pub struct ParallaxBackgroundSprite {
    pub idx: i32,
    pub meta: ParallaxLayerMeta,
}

#[derive(Clone, Debug, TypeUlid, Default)]
#[ulid = "01GPV6M1KY0GBRQVJ3WG5CSBBS"]
pub struct CameraState {
    pub player_camera_rects: [Rect; MAX_PLAYERS],
}

fn camera_controller(
    game_meta: Res<CoreMetaArc>,
    entities: Res<Entities>,
    map_handle: Res<MapHandle>,
    map_assets: BevyAssets<MapMeta>,
    mut cameras: CompMut<Camera>,
    mut camera_shakes: CompMut<CameraShake>,
    mut camera_states: CompMut<CameraState>,
    transforms: Comp<Transform>,
    player_indexes: Comp<PlayerIdx>,
    bodies: Comp<KinematicBody>,
    window: Res<Window>,
) {
    let meta = &game_meta.camera;

    let Some(map) = map_assets.get(&map_handle.get_bevy_handle()) else {
        return;
    };
    let Some((_ent, (camera, camera_shake, camera_state))) = entities.iter_with((&mut cameras, &mut camera_shakes, &mut camera_states)).next() else {
        return
    };

    // Update player camera rects
    for (_ent, (transform, player_idx, body)) in
        entities.iter_with((&transforms, &player_indexes, &bodies))
    {
        let camera_box_size = meta.player_camera_box_size;

        // Get the player's camera box
        let camera_box = &mut camera_state.player_camera_rects[player_idx.0];

        // If it's not be initialized.
        if camera_box.min == Vec2::ZERO && camera_box.max == Vec2::ZERO {
            // Set it's size and position according to the player
            *camera_box = Rect::new(
                transform.translation.x,
                transform.translation.y,
                camera_box_size.x,
                camera_box_size.y,
            );
        }

        // Get the player's body rect
        let rect = body.bounding_box(*transform);

        // Push the camera rect just enough to contain the player's body rect
        if rect.min.x < camera_box.min.x {
            camera_box.min.x = rect.min.x;
            camera_box.max.x = camera_box.min.x + camera_box_size.x;
        }
        if rect.max.x > camera_box.max.x {
            camera_box.max.x = rect.max.x;
            camera_box.min.x = rect.max.x - camera_box_size.x;
        }
        if rect.min.y < camera_box.min.y {
            camera_box.min.y = rect.min.y;
            camera_box.max.y = camera_box.min.y + camera_box_size.y;
        }
        if rect.max.y > camera_box.max.y {
            camera_box.max.y = rect.max.y;
            camera_box.min.y = rect.max.y - camera_box_size.y;
        }
    }

    let window_aspect = window.size.x / window.size.y;
    let default_height = meta.default_height;
    let mut scale = camera.height / default_height;
    let default_width = window_aspect * default_height;
    let map_size = map.grid_size.as_vec2() * map.tile_size;

    let mut min = Vec2::new(f32::MAX, f32::MAX);
    let mut max = Vec2::new(f32::MIN, f32::MIN);

    let players: Vec<usize> = entities
        .iter_with(&player_indexes)
        .map(|x| x.1 .0)
        .collect();
    let player_count = players.len();

    for player_idx in players {
        let rect = camera_state.player_camera_rects[player_idx];

        min = (rect.min - vec2(meta.border_left, meta.border_bottom))
            .min(min)
            .max(Vec2::ZERO);
        max = (rect.max + vec2(meta.border_right, meta.border_top)).max(max);
        max.x = max.x.min(map_size.x)
    }

    let camera_pos = &mut camera_shake.center;

    let mut middle_point = if player_count == 0 {
        camera_pos.truncate()
    } else {
        Rect { min, max }.center()
    };

    let size = max - min;
    let size = size.max(meta.min_camera_size);

    let rh = size.y / default_height;
    let rw = size.x / default_width;
    let r_target = if rh > rw { rh } else { rw };
    let r_diff = r_target - scale;
    if r_diff > 0.0 {
        scale += r_diff * meta.zoom_out_lerp_factor;
    } else {
        scale += r_diff * meta.zoom_in_lerp_factor;
    }

    // Keep camera above the map floor
    if middle_point.y - size.y / 2. < 0.0 {
        middle_point.y = size.y / 2.0;
    }

    let delta = camera_pos.truncate() - middle_point;
    let dist = delta * meta.move_lerp_factor;
    camera.height = scale * default_height;
    *camera_pos -= dist.extend(0.0);
}

fn camera_parallax(
    entities: Res<Entities>,
    mut transforms: CompMut<Transform>,
    parallax_bg_sprites: Comp<ParallaxBackgroundSprite>,
    cameras: Comp<Camera>,
    map_handle: Res<MapHandle>,
    map_assets: BevyAssets<MapMeta>,
) {
    // TODO: This constant represents that maximum camera-visible distance, and should be moved
    // somewhere more appropriate.
    const FAR_PLANE: f32 = 1000.0;

    let Some(map) = map_assets.get(&map_handle.get_bevy_handle()) else {
        return;
    };
    let map_size = map.grid_size.as_vec2() * map.tile_size;

    let camera_transform = entities
        .iter_with((&transforms, &cameras))
        .next()
        .map(|x| x.1 .0)
        .copied()
        .unwrap();
    let camera_offset = map_size / 2.0 - camera_transform.translation.truncate();

    for (_ent, (transform, bg)) in entities.iter_with((&mut transforms, &parallax_bg_sprites)) {
        transform.scale.x = bg.meta.scale;
        transform.scale.y = bg.meta.scale;
        let display_size = transform.scale.truncate() * bg.meta.size;
        transform.translation.x = bg.idx as f32 * display_size.x;
        transform.translation.y = map_size.y / 2.0;
        transform.translation.z = -FAR_PLANE + 1.0 - bg.meta.depth / 100.0;
        transform.translation += bg.meta.offset.extend(1.0);

        transform.translation.x -= camera_offset.x * bg.meta.depth * map.background.speed.x;
        transform.translation.y += camera_offset.y * bg.meta.depth * map.background.speed.y;
    }
}
