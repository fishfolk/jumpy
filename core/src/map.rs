use crate::prelude::{collisions::TileCollisionKind, *};

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::First, spawn_map)
        .add_system_to_stage(CoreStage::First, handle_out_of_bounds_players_and_items);
}

/// Resource containing the map metadata for this game session.
#[derive(Clone, TypeUlid, Deref, DerefMut, Default)]
#[ulid = "01GP2H6K9H3JEEMXFCKV4TGMWZ"]
pub struct LoadedMap(pub Arc<MapMeta>);

/// Resource indicating whether the map has been spawned.
#[derive(Clone, TypeUlid, Default, Deref, DerefMut)]
#[ulid = "01GP3Z38HKE37JB6GRHHPPTY38"]
pub struct MapSpawned(pub bool);

#[derive(Clone, TypeUlid, Default, Deref, DerefMut)]
#[ulid = "01GP9NY0Y50Y2A8M4A7E9NN8VE"]
pub struct MapRespawnPoint(pub Vec3);

/// Helper for getting the z-depth of the map layer with the given index.
pub fn z_depth_for_map_layer(layer_idx: usize) -> f32 {
    // We start map layers at -900 and for ever layer we place a gap of 2 units in between
    -900.0 + layer_idx as f32 * 2.0
}

/// Resource containing essential the map metadata for the map once spawned. This allows the
/// complete map metadata to be re-constructed from the world after the map has been spawned and
/// potentially modified.
#[derive(TypeUlid, Clone)]
#[ulid = "01GSR8V683B3EH5QAB2PMGN9J7"]
pub struct SpawnedMapMeta {
    pub name: Arc<str>,
    pub background: Arc<BackgroundMeta>,
    pub background_color: ColorMeta,
    pub grid_size: UVec2,
    pub tile_size: Vec2,
    pub layer_names: Arc<[String]>,
}

impl Default for SpawnedMapMeta {
    fn default() -> Self {
        Self {
            name: "".into(),
            background: default(),
            background_color: default(),
            grid_size: default(),
            tile_size: default(),
            layer_names: Arc::new([]),
        }
    }
}

/// Component containing the map layer that an entity is associated to.
///
/// This is used when exporting the world to `MapMeta` to decide which layer to put an element or
/// tile layer in.
#[derive(TypeUlid, Clone, Copy, Default)]
#[ulid = "01GSR8GSRJHGTJ8J9Y38W7C5S3"]
pub struct SpawnedMapLayerMeta {
    /// The layer index of the layer that the element belongs to in the map.
    pub layer_idx: usize,
}

fn spawn_map(
    mut commands: Commands,
    mut entities: ResMut<Entities>,
    mut clear_color: ResMut<ClearColor>,
    map: Res<LoadedMap>,
    mut map_spawned: ResMut<MapSpawned>,
    mut tiles: CompMut<Tile>,
    mut tile_layers: CompMut<TileLayer>,
    mut transforms: CompMut<Transform>,
    mut element_handles: CompMut<ElementHandle>,
    mut tile_collisions: CompMut<TileCollisionKind>,
    mut parallax_bg_sprites: CompMut<ParallaxBackgroundSprite>,
    mut sprites: CompMut<Sprite>,
    mut cameras: CompMut<Camera>,
    mut camera_shakes: CompMut<CameraShake>,
    mut camera_states: CompMut<CameraState>,
    mut spawned_map_layer_metas: CompMut<SpawnedMapLayerMeta>,
    mut spawned_map_meta: ResMut<SpawnedMapMeta>,
) {
    if map_spawned.0 {
        return;
    }

    // Fill in the spawned map metadata
    *spawned_map_meta = SpawnedMapMeta {
        name: map.name.clone().into(),
        background: Arc::new(map.background.clone()),
        background_color: map.background_color,
        grid_size: map.grid_size,
        tile_size: map.tile_size,
        layer_names: map.layers.iter().map(|x| x.id.to_string()).collect(),
    };

    // Spawn the camera
    {
        let ent = entities.create();
        camera_shakes.insert(
            ent,
            CameraShake {
                center: (map.tile_size * (map.grid_size / 2).as_vec2()).extend(0.0),
                ..CameraShake::new(6.0, glam::vec2(3.0, 3.0), 1.0)
            },
        );
        cameras.insert(ent, default());
        transforms.insert(ent, default());
        camera_states.insert(ent, default());
    }

    map_spawned.0 = true;
    **clear_color = map.background_color.0;

    // Spawn parallax backgrounds
    for layer in &map.background.layers {
        for i in -1..=1 {
            let ent = entities.create();
            sprites.insert(
                ent,
                Sprite {
                    image: layer.image.clone(),
                    ..default()
                },
            );
            transforms.insert(ent, default());
            parallax_bg_sprites.insert(
                ent,
                ParallaxBackgroundSprite {
                    idx: i,
                    meta: layer.clone(),
                },
            );
        }
    }

    // Load tiles
    for (layer_idx, layer) in map.layers.iter().enumerate() {
        let layer_z = z_depth_for_map_layer(layer_idx);
        if let Some(atlas) = layer.tilemap.clone() {
            let mut tile_layer = TileLayer::new(map.grid_size, map.tile_size, atlas);

            for tile_meta in &layer.tiles {
                let tile_ent = entities.create();
                tile_layer.set(tile_meta.pos, Some(tile_ent));
                tiles.insert(
                    tile_ent,
                    Tile {
                        idx: tile_meta.idx as usize,
                        ..default()
                    },
                );
                tile_collisions.insert(
                    tile_ent,
                    if tile_meta.jump_through {
                        TileCollisionKind::JUMP_THROUGH
                    } else {
                        TileCollisionKind::SOLID
                    },
                );
            }
            let layer_ent = entities.create();
            spawned_map_layer_metas.insert(layer_ent, SpawnedMapLayerMeta { layer_idx });
            tile_layers.insert(layer_ent, tile_layer);
            transforms.insert(
                layer_ent,
                Transform::from_translation(Vec3::new(0.0, 0.0, layer_z)),
            );
        }

        for element_meta in &layer.elements {
            let element_ent = entities.create();

            spawned_map_layer_metas.insert(element_ent, SpawnedMapLayerMeta { layer_idx });
            transforms.insert(
                element_ent,
                Transform::from_translation(element_meta.pos.extend(layer_z)),
            );
            element_handles.insert(element_ent, ElementHandle(element_meta.element.clone()));
        }
    }

    // Update collision world with map tiles
    commands.add(|mut collision_world: CollisionWorld| {
        collision_world.update_tiles();
    });
}

fn handle_out_of_bounds_players_and_items(
    entities: Res<Entities>,
    mut transforms: CompMut<Transform>,
    player_indexes: Comp<PlayerIdx>,
    map: Res<LoadedMap>,
    mut player_events: ResMut<PlayerEvents>,
    map_respawn_points: Comp<MapRespawnPoint>,
) {
    const KILL_ZONE_BORDER: f32 = 500.0;

    let map_width = map.grid_size.x as f32 * map.tile_size.x;
    let left_kill_zone = -KILL_ZONE_BORDER;
    let right_kill_zone = map_width + KILL_ZONE_BORDER;
    let bottom_kill_zone = -KILL_ZONE_BORDER;

    // Kill out of bounds players
    for (player_ent, (_player_idx, transform)) in entities.iter_with((&player_indexes, &transforms))
    {
        let pos = transform.translation;

        if pos.x < left_kill_zone || pos.x > right_kill_zone || pos.y < bottom_kill_zone {
            player_events.kill(player_ent, None);
        }
    }

    // Reset out of bound item positions
    for (_ent, (respawn_point, transform)) in
        entities.iter_with((&map_respawn_points, &mut transforms))
    {
        let pos = transform.translation;

        if pos.x < left_kill_zone || pos.x > right_kill_zone || pos.y < bottom_kill_zone {
            transform.translation = respawn_point.0;
        }
    }
}
