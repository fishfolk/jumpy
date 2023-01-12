use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    // Spawn the camera
    session
        .world
        .run_system(
            |mut entities: ResMut<Entities>,
             mut cameras: CompMut<Camera>,
             mut transforms: CompMut<Transform>| {
                let ent = entities.create();

                cameras.insert(ent, default());
                transforms.insert(ent, default());
            },
        )
        .unwrap();

    session
        .stages
        .add_system_to_stage(CoreStage::Last, camera_controller);
}

fn camera_controller(
    meta: Res<CoreMetaArc>,
    entities: Res<Entities>,
    map_handle: Res<MapHandle>,
    map_assets: BevyAssets<MapMeta>,
    mut cameras: CompMut<Camera>,
    mut transforms: CompMut<Transform>,
    player_indexes: Comp<PlayerIdx>,
    window: Res<Window>,
) {
    const CAMERA_PADDING: f32 = 300.0;
    const MOVE_LERP_FACTOR: f32 = 0.1;
    const ZOOM_IN_LERP_FACTOR: f32 = 0.04;
    const ZOOM_OUT_LERP_FACTOR: f32 = 0.1;
    const MIN_BOUND: f32 = 350.0;

    let Some(map) = map_assets.get(&map_handle.get_bevy_handle()) else {
        return;
    };
    let Some((camera_ent, (camera, camera_transform))) = entities.iter_with((&mut cameras, &mut transforms)).next() else {
        return
    };
    let mut camera_transform = *camera_transform;

    let window_aspect = window.size.x / window.size.y;
    let default_height = meta.camera_height;
    let mut scale = camera.height / default_height;
    let default_width = window_aspect * default_height;

    let map_width = map.tile_size.x * map.grid_size.x as f32;
    let map_height = map.tile_size.y * map.grid_size.y as f32;
    let min_player_pos = Vec2::new(-CAMERA_PADDING, -CAMERA_PADDING);
    let max_player_pos = Vec2::new(map_width + CAMERA_PADDING, map_height + CAMERA_PADDING);

    let mut middle_point = Vec2::ZERO;
    let mut min = Vec2::new(100000.0, 100000.0);
    let mut max = Vec2::new(-100000.0, -100000.0);

    let players: Vec<&Transform> = entities
        .iter_with((&player_indexes, &transforms))
        .map(|x| x.1 .1)
        .collect();

    let player_count = players.len();

    for player_transform in &players {
        let pos = player_transform.translation.truncate();
        let pos = pos.max(min_player_pos).min(max_player_pos);
        middle_point += pos;

        min.x = pos.x.min(min.x);
        min.y = pos.y.min(min.y);
        max.x = pos.x.max(max.x);
        max.y = pos.y.max(max.y);
    }

    middle_point /= player_count.max(1) as f32;

    let size = (max - min) + CAMERA_PADDING;
    let size = size.max(Vec2::splat(MIN_BOUND));

    let rh = size.y / default_height;
    let rw = size.x / default_width;
    let r_target = if rh > rw { rh } else { rw };
    let r_diff = r_target - scale;
    // We zoom out twice as fast as we zoom in
    if r_diff > 0.0 {
        scale += r_diff * ZOOM_OUT_LERP_FACTOR;
    } else {
        scale += r_diff * ZOOM_IN_LERP_FACTOR;
    }

    let delta = camera_transform.translation.truncate() - middle_point;
    let dist = delta * MOVE_LERP_FACTOR;
    camera.height = scale * default_height;
    camera_transform.translation -= dist.extend(0.0);
    transforms.insert(camera_ent, camera_transform);
}
