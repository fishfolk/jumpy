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
pub struct MapHandle(pub Handle<MapMeta>);

/// Resource indicating whether the map has been spawned.
#[derive(Clone, TypeUlid, Default, Deref, DerefMut)]
#[ulid = "01GP3Z38HKE37JB6GRHHPPTY38"]
pub struct MapSpawned(pub bool);

#[derive(Clone, TypeUlid, Default, Deref, DerefMut)]
#[ulid = "01GP9NY0Y50Y2A8M4A7E9NN8VE"]
pub struct MapRespawnPoint(pub Vec3);

fn spawn_map(
    mut commands: Commands,
    mut entities: ResMut<Entities>,
    mut clear_color: ResMut<ClearColor>,
    map_handle: Res<MapHandle>,
    map_assets: BevyAssets<MapMeta>,
    mut map_spawned: ResMut<MapSpawned>,
    mut tiles: CompMut<Tile>,
    mut tile_layers: CompMut<TileLayer>,
    mut transforms: CompMut<Transform>,
    mut element_handles: CompMut<ElementHandle>,
    mut tile_collisions: CompMut<TileCollisionKind>,
    mut parallax_bg_sprites: CompMut<ParallaxBackgroundSprite>,
    mut sprites: CompMut<Sprite>,
) {
    if map_spawned.0 {
        return;
    }
    let Some(map) = map_assets.get(&map_handle.get_bevy_handle()) else {
        return;
    };
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
    for (i, layer) in map.layers.iter().enumerate() {
        let layer_z = -900.0 + i as f32;
        match &layer.kind {
            MapLayerKind::Tile(tile_layer_meta) => {
                let mut tile_layer = TileLayer::new(
                    map.grid_size,
                    map.tile_size,
                    tile_layer_meta.tilemap.clone(),
                );

                for tile_meta in &tile_layer_meta.tiles {
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
                tile_layers.insert(layer_ent, tile_layer);
                transforms.insert(
                    layer_ent,
                    Transform::from_translation(Vec3::new(0.0, 0.0, layer_z)),
                );
            }
            MapLayerKind::Element(element_layer_meta) => {
                for element_meta in &element_layer_meta.elements {
                    let element_ent = entities.create();

                    transforms.insert(
                        element_ent,
                        Transform::from_translation(element_meta.pos.extend(layer_z)),
                    );
                    element_handles
                        .insert(element_ent, ElementHandle(element_meta.element.clone()));
                }
            }
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
    map_handle: Res<MapHandle>,
    map_assets: BevyAssets<MapMeta>,
    mut player_events: ResMut<PlayerEvents>,
    map_respawn_points: Comp<MapRespawnPoint>,
) {
    const KILL_ZONE_BORDER: f32 = 500.0;
    let Some(map) = map_assets.get(&map_handle.get_bevy_handle()) else {
        return;
    };

    let map_width = map.grid_size.x as f32 * map.tile_size.x;
    let left_kill_zone = -KILL_ZONE_BORDER;
    let right_kill_zone = map_width + KILL_ZONE_BORDER;
    let bottom_kill_zone = -KILL_ZONE_BORDER;

    // Kill out of bounds players
    for (player_ent, (_player_idx, transform)) in entities.iter_with((&player_indexes, &transforms))
    {
        let pos = transform.translation;

        if pos.x < left_kill_zone || pos.x > right_kill_zone || pos.y < bottom_kill_zone {
            player_events.kill(player_ent);
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
