use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    // Spawn the camera and parallax background.
    session
        .world
        .run_system(
            |mut entities: ResMut<Entities>,
             mut cameras: CompMut<Camera>,
             mut transforms: CompMut<Transform>,
             mut camera_shakes: CompMut<CameraShake>| {
                let ent = entities.create();

                camera_shakes.insert(ent, CameraShake::new(6.0, glam::vec2(3.0, 3.0), 1.0));
                cameras.insert(ent, default());
                transforms.insert(ent, default());
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

fn camera_controller(
    meta: Res<CoreMetaArc>,
    entities: Res<Entities>,
    map_handle: Res<MapHandle>,
    map_assets: BevyAssets<MapMeta>,
    mut cameras: CompMut<Camera>,
    transforms: Comp<Transform>,
    mut camera_shakes: CompMut<CameraShake>,
    player_indexes: Comp<PlayerIdx>,
    window: Res<Window>,
) {
    const CAMERA_PADDING: f32 = 120.0;
    const MOVE_LERP_FACTOR: f32 = 0.08;
    const ZOOM_IN_LERP_FACTOR: f32 = 0.01;
    const ZOOM_OUT_LERP_FACTOR: f32 = 0.1;
    const BELOW_GROUND_VIEW: f32 = 100.0;
    const MIN_SIZE: f32 = 530.0;

    let Some(map) = map_assets.get(&map_handle.get_bevy_handle()) else {
        return;
    };
    let Some((_ent, (camera, camera_shake))) = entities.iter_with((&mut cameras, &mut camera_shakes)).next() else {
        return
    };
    let camera_pos = &mut camera_shake.center;

    let window_aspect = window.size.x / window.size.y;
    let default_height = meta.camera_height;
    let mut scale = camera.height / default_height;
    let default_width = window_aspect * default_height;

    let map_width = map.tile_size.x * map.grid_size.x as f32;
    let map_height = map.tile_size.y * map.grid_size.y as f32;
    let min_player_pos = Vec2::new(-CAMERA_PADDING, CAMERA_PADDING - BELOW_GROUND_VIEW);
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

        min.x = (pos.x - CAMERA_PADDING).min(min.x);
        min.y = (pos.y - CAMERA_PADDING).min(min.y);
        max.x = (pos.x + CAMERA_PADDING).max(max.x);
        max.y = (pos.y + CAMERA_PADDING).max(max.y);
    }

    middle_point /= player_count.max(1) as f32;

    let size = max - min;
    let size = size.max(Vec2::splat(MIN_SIZE));

    let rh = size.y / default_height;
    let rw = size.x / default_width;
    let r_target = if rh > rw { rh } else { rw };
    let r_diff = r_target - scale;
    if r_diff > 0.0 {
        scale += r_diff * ZOOM_OUT_LERP_FACTOR;
    } else {
        scale += r_diff * ZOOM_IN_LERP_FACTOR;
    }

    let delta = camera_pos.truncate() - middle_point;
    let dist = delta * MOVE_LERP_FACTOR;
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
