use crate::{map::z_depth_for_map_layer, prelude::*};

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, handle_editor_input);
}

fn handle_editor_input(
    mut commands: Commands,
    player_inputs: Res<PlayerInputs>,
    mut entities: ResMut<Entities>,
    mut spawned_map_meta: ResMut<SpawnedMapMeta>,
    mut element_handles: CompMut<ElementHandle>,
    mut transforms: CompMut<Transform>,
    mut spawned_map_layer_metas: CompMut<SpawnedMapLayerMeta>,
    mut tile_layers: CompMut<TileLayer>,
    mut tiles: CompMut<Tile>,
    mut tile_collisions: CompMut<TileCollisionKind>,
) {
    for player in &player_inputs.players {
        if let Some(editor_input) = &player.editor_input {
            match editor_input {
                EditorInput::SpawnElement {
                    handle,
                    translation,
                    layer,
                } => {
                    let ent = entities.create();
                    element_handles.insert(ent, ElementHandle(handle.clone()));
                    let z_depth = z_depth_for_map_layer(*layer as usize);
                    transforms.insert(
                        ent,
                        Transform::from_translation(translation.extend(z_depth)),
                    );
                    spawned_map_layer_metas.insert(
                        ent,
                        SpawnedMapLayerMeta {
                            layer_idx: *layer as usize,
                        },
                    );
                }
                EditorInput::CreateLayer { id } => {
                    let ent = entities.create();
                    let layer_idx = spawned_map_meta.layer_names.len();
                    spawned_map_meta.layer_names = spawned_map_meta
                        .layer_names
                        .clone()
                        .iter()
                        .cloned()
                        .chain([id.clone()])
                        .collect();
                    spawned_map_layer_metas.insert(ent, SpawnedMapLayerMeta { layer_idx });
                    tile_layers.insert(
                        ent,
                        TileLayer::new(
                            spawned_map_meta.grid_size,
                            spawned_map_meta.tile_size,
                            default(),
                        ),
                    );
                    transforms.insert(
                        ent,
                        Transform::from_translation(Vec3::new(
                            0.0,
                            0.0,
                            z_depth_for_map_layer(layer_idx),
                        )),
                    );
                }
                EditorInput::DeleteLayer { layer } => {
                    let layer = *layer as usize;
                    let layer_count = spawned_map_meta.layer_names.len();
                    let layers_to_decrement = layer_count - layer;
                    spawned_map_meta.layer_names = spawned_map_meta
                        .layer_names
                        .clone()
                        .iter()
                        .cloned()
                        .enumerate()
                        .filter_map(|(i, name)| if i == layer { None } else { Some(name) })
                        .collect();
                    let mut to_kill = Vec::new();
                    entities
                        .iter_with(&mut spawned_map_layer_metas)
                        .for_each(|(ent, l)| {
                            if l.layer_idx == layer {
                                to_kill.push(ent);

                                if let Some(tile_layer) = tile_layers.get(ent) {
                                    for tile in tile_layer.tiles.iter().flatten() {
                                        to_kill.push(*tile);
                                    }
                                }
                            };

                            if l.layer_idx > layer_count - layers_to_decrement {
                                l.layer_idx -= 1;
                            }
                        });
                    to_kill.into_iter().for_each(|ent| {
                        entities.kill(ent);
                    });
                }
                EditorInput::RenameLayer {
                    layer,
                    name: new_name,
                } => {
                    let layer = *layer as usize;

                    spawned_map_meta.layer_names = spawned_map_meta
                        .layer_names
                        .clone()
                        .iter()
                        .cloned()
                        .enumerate()
                        .map(|(i, name)| if i == layer { new_name.clone() } else { name })
                        .collect();
                }
                EditorInput::MoveEntity { entity, pos } => {
                    let transform = transforms.get_mut(*entity).unwrap();
                    transform.translation.x = pos.x;
                    transform.translation.y = pos.y;
                }
                EditorInput::DeleteEntity { entity } => {
                    entities.kill(*entity);
                }
                EditorInput::SetTilemap { layer, handle } => {
                    if let Some((_ent, (tile_layer, _))) = entities
                        .iter_with((&mut tile_layers, &spawned_map_layer_metas))
                        .find(|x| x.1 .1.layer_idx == *layer as usize)
                    {
                        if let Some(handle) = handle {
                            tile_layer.atlas = handle.clone();
                        } else {
                            tile_layer.atlas = default();
                        }
                    };
                }
                EditorInput::SetTile {
                    layer,
                    pos,
                    tilemap_tile_idx,
                    collision,
                } => {
                    let layer = *layer as usize;
                    let pos = *pos;

                    if let Some((_ent, (tile_layer, _))) = entities
                        .iter_with((&mut tile_layers, &spawned_map_layer_metas))
                        .find(|x| x.1 .1.layer_idx == layer)
                    {
                        if let Some(ent) = tile_layer.get(pos) {
                            if let Some(idx) = tilemap_tile_idx.as_ref() {
                                tiles.get_mut(ent).unwrap().idx = *idx;

                                // TODO: technically setting the collision to empty should be
                                // equivalent to removing the component, but it isn't working like
                                // that right now.
                                if *collision != TileCollisionKind::Empty {
                                    tile_collisions.insert(ent, *collision);
                                } else {
                                    tile_collisions.remove(ent);
                                }
                            } else {
                                entities.kill(ent);
                                tile_layer.set(pos, None);
                            }
                        } else if let Some(idx) = tilemap_tile_idx {
                            let ent = entities.create();
                            tile_layer.set(pos, Some(ent));
                            tiles.insert(
                                ent,
                                Tile {
                                    idx: *idx,
                                    ..default()
                                },
                            );

                            // TODO: technically setting the collision to empty should be
                            // equivalent to removing the component, but it isn't workidesired_keyng like
                            // that right now.
                            if *collision != TileCollisionKind::Empty {
                                tile_collisions.insert(ent, *collision);
                            } else {
                                tile_collisions.remove(ent);
                            }
                        }

                        commands.add(move |mut collision_world: CollisionWorld| {
                            collision_world.update_tile(layer, pos);
                        });
                    };
                }
                EditorInput::MoveLayer { layer, down } => {
                    let layer1 = *layer as usize;
                    let layer2 = if *down { layer1 + 1 } else { layer1 - 1 };
                    let mut layer_names = spawned_map_meta.layer_names.to_vec();
                    layer_names.swap(layer1, layer2);
                    spawned_map_meta.layer_names = layer_names.into_iter().collect();

                    for (_ent, (transform, layer_meta)) in
                        entities.iter_with((&mut transforms, &mut spawned_map_layer_metas))
                    {
                        if layer_meta.layer_idx == layer1 {
                            layer_meta.layer_idx = layer2;
                            transform.translation.z = z_depth_for_map_layer(layer2);
                        } else if layer_meta.layer_idx == layer2 {
                            layer_meta.layer_idx = layer1;
                            transform.translation.z = z_depth_for_map_layer(layer1);
                        }
                    }
                }
            }
        }
    }
}
