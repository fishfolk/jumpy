//! Map editing implementation.
//!
//! Allows you to edit the game map while the game is running.

use crate::core::map_constructor::{shiftnanigans::ShiftnanigansMapConstructor, MapConstructor};
use crate::prelude::*;

/// Install this module.
pub fn install(session: &mut SessionBuilder) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, handle_editor_input);
}

impl_system_param! {
    /// A system parameter for editing the map.
    ///
    /// [`MapManager`] provides an interface for editing the map contents easily while the match is
    /// running. This can be used both for manual map editing, and by algorithms for generating or
    /// randomizing maps.
    ///
    /// Map generators implement the [`MapConstructor`]
    /// trait, which is given a [`MapManager`] to make its changes with.
    pub struct MapManager<'a> {
        commands: Commands<'a>,
        entities: ResMutInit<'a, Entities>,
        spawned_map_meta: ResMutInit<'a, SpawnedMapMeta>,
        element_handles: CompMut<'a, ElementHandle>,
        transforms: CompMut<'a, Transform>,
        spawned_map_layer_metas: CompMut<'a, SpawnedMapLayerMeta>,
        tile_layers: CompMut<'a, TileLayer>,
        tiles: CompMut<'a, Tile>,
        tile_collisions: CompMut<'a, TileCollisionKind>,
        map: Res<'a, LoadedMap>,
        element_kill_callbacks: Comp<'a, ElementKillCallback>,
        spawner_manager: SpawnerManager<'a>,
    }
}

impl<'a> MapManager<'a> {
    /// Create a new map element at the given location on the given layer.
    pub fn create_element(
        &mut self,
        element_meta_handle: &Handle<ElementMeta>,
        translation: &Vec2,
        layer_index: u32,
    ) {
        let entity = self.entities.create();
        // TODO remove element handles as the underlying elements are removed
        self.element_handles
            .insert(entity, ElementHandle(*element_meta_handle));
        let z_depth = z_depth_for_map_layer(layer_index);
        self.transforms.insert(
            entity,
            Transform::from_translation(translation.extend(z_depth)),
        );
        self.spawned_map_layer_metas.insert(
            entity,
            SpawnedMapLayerMeta {
                layer_idx: layer_index,
            },
        );
    }
    /// Create a new layer with the given name.
    pub fn create_layer(&mut self, name: Ustr) {
        let entity = self.entities.create();
        let layer_index = self.spawned_map_meta.layer_names.len() as u32;
        self.spawned_map_meta.layer_names = self
            .spawned_map_meta
            .layer_names
            .clone()
            .iter()
            .cloned()
            .chain([name])
            .collect();
        self.spawned_map_layer_metas.insert(
            entity,
            SpawnedMapLayerMeta {
                layer_idx: layer_index,
            },
        );
        self.tile_layers.insert(
            entity,
            TileLayer::new(
                self.spawned_map_meta.grid_size,
                self.spawned_map_meta.tile_size,
                default(),
            ),
        );
        self.transforms.insert(
            entity,
            Transform::from_translation(Vec3::new(0.0, 0.0, z_depth_for_map_layer(layer_index))),
        );
    }
    /// Delete the layer with the given index.
    pub fn delete_layer(&mut self, layer_index: u32) {
        let layer_count = self.spawned_map_meta.layer_names.len() as u32;
        let layers_to_decrement = layer_count - layer_index;
        self.spawned_map_meta.layer_names = self
            .spawned_map_meta
            .layer_names
            .clone()
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(i, name)| {
                if i as u32 == layer_index {
                    None
                } else {
                    Some(name)
                }
            })
            .collect();
        let mut to_kill = Vec::new();
        self.entities
            .iter_with(&mut self.spawned_map_layer_metas)
            .for_each(|(entity, layer)| {
                if layer.layer_idx == layer_index {
                    to_kill.push(entity);

                    if let Some(tile_layer) = self.tile_layers.get(entity) {
                        for tile in tile_layer.tiles.iter().flatten() {
                            to_kill.push(*tile);
                        }
                    }
                };

                if layer.layer_idx > layer_count - layers_to_decrement {
                    layer.layer_idx -= 1;
                }
            });
        to_kill.into_iter().for_each(|ent| {
            self.entities.kill(ent);
        });
    }
    /// Rename the layer with the given index.
    pub fn rename_layer(&mut self, layer_index: u32, name: &str) {
        self.spawned_map_meta.layer_names = self
            .spawned_map_meta
            .layer_names
            .clone()
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, n)| {
                if i as u32 == layer_index {
                    ustr(name)
                } else {
                    n
                }
            })
            .collect();
    }
    /// Move an element to a new position on the map.
    pub fn move_element(&mut self, entity: Entity, position: &Vec2) {
        let transform = self.transforms.get_mut(entity).unwrap();
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
    /// Delete an element off of the map.
    pub fn delete_element(&mut self, entity: Entity) {
        if let Some(element_kill_callback) = self.element_kill_callbacks.get(entity) {
            let system = element_kill_callback.system.clone();
            self.commands
                .add(move |world: &World| (system.lock().unwrap().run)(world, ()));
        } else if self.spawner_manager.is_entity_a_spawner(entity) {
            // if the entity is a spawner, there are specific rules around how child entities may be killed
            self.spawner_manager.kill_spawner_entity(
                entity,
                &mut self.entities,
                &self.element_kill_callbacks,
                &mut self.commands,
            );
        } else {
            self.entities.kill(entity);
        }
    }
    /// Set the tilemap for the given layer.
    pub fn set_layer_tilemap(&mut self, layer_index: u32, tilemap: &Option<Handle<Atlas>>) {
        if let Some((_, (tile_layer, _))) = self
            .entities
            .iter_with((&mut self.tile_layers, &self.spawned_map_layer_metas))
            .find(|x| x.1 .1.layer_idx == layer_index)
        {
            if let Some(handle) = tilemap {
                tile_layer.atlas = *handle;
            } else {
                tile_layer.atlas = default();
            }
        };
    }
    /// Set the tile index of a tile on the given layer.
    pub fn set_tile(
        &mut self,
        layer_index: u32,
        position: UVec2,
        tilemap_tile_index: &Option<u32>,
        tile_collision_kind: TileCollisionKind,
    ) {
        if let Some((_, (tile_layer, _))) = self
            .entities
            .iter_with((&mut self.tile_layers, &self.spawned_map_layer_metas))
            .find(|x| x.1 .1.layer_idx == layer_index)
        {
            if let Some(entity) = tile_layer.get(position) {
                if let Some(idx) = tilemap_tile_index.as_ref() {
                    self.tiles.get_mut(entity).unwrap().idx = *idx;

                    // TODO: technically setting the collision to empty should be
                    // equivalent to removing the component, but it isn't working like
                    // that right now.
                    if tile_collision_kind != TileCollisionKind::Empty {
                        self.tile_collisions.insert(entity, tile_collision_kind);
                    } else {
                        self.tile_collisions.remove(entity);
                    }
                } else {
                    self.entities.kill(entity);
                    tile_layer.set(position, None);
                }
            } else if let Some(idx) = tilemap_tile_index {
                let entity = self.entities.create();
                tile_layer.set(position, Some(entity));
                self.tiles.insert(
                    entity,
                    Tile {
                        idx: *idx,
                        ..default()
                    },
                );

                // TODO: technically setting the collision to empty should be
                // equivalent to removing the component, but it isn't workidesired_keyng like
                // that right now.
                if tile_collision_kind != TileCollisionKind::Empty {
                    self.tile_collisions.insert(entity, tile_collision_kind);
                } else {
                    self.tile_collisions.remove(entity);
                }
            }

            self.commands
                .add(move |mut collision_world: CollisionWorld| {
                    collision_world.update_tile(layer_index, position);
                });
        };
    }
    /// Swap the position of two layers.
    pub fn swap_layer(&mut self, layer_index: u32, is_downward: bool) {
        let origin_layer_index = layer_index;
        let other_layer_index = if is_downward {
            origin_layer_index + 1
        } else {
            origin_layer_index - 1
        };
        let mut layer_names = self.spawned_map_meta.layer_names.to_vec();
        layer_names.swap(origin_layer_index as usize, other_layer_index as usize);
        self.spawned_map_meta.layer_names = layer_names.into_iter().collect();

        for (_, (transform, layer_meta)) in self
            .entities
            .iter_with((&mut self.transforms, &mut self.spawned_map_layer_metas))
        {
            if layer_meta.layer_idx == origin_layer_index {
                layer_meta.layer_idx = other_layer_index;
                transform.translation.z = z_depth_for_map_layer(other_layer_index);
            } else if layer_meta.layer_idx == other_layer_index {
                layer_meta.layer_idx = origin_layer_index;
                transform.translation.z = z_depth_for_map_layer(origin_layer_index);
            }
        }
    }
    /// Rename the map.
    pub fn rename_map(&mut self, name: &str) {
        self.spawned_map_meta.name = ustr(name);
    }
    /// Get the size of the map.
    pub fn get_size(&self) -> UVec2 {
        self.spawned_map_meta.grid_size
    }
    /// The the number of layers.
    pub fn get_layers_total(&self) -> usize {
        self.spawned_map_meta.layer_names.len()
    }
    /// Clear all the tiles on the map.
    pub fn clear_tiles(&mut self) {
        let empty_tile: Option<u32> = Option::None;
        for y in 0..self.spawned_map_meta.grid_size.y {
            for x in 0..self.spawned_map_meta.grid_size.x {
                let position = UVec2 { x, y };
                for layer_index in 0..self.spawned_map_meta.layer_names.len() as u32 {
                    self.set_tile(layer_index, position, &empty_tile, TileCollisionKind::Empty);
                }
            }
        }
    }
    /// Clear all of the elements on the map.
    pub fn clear_elements(&mut self) {
        let mut to_kill: Vec<Entity> = Vec::new();
        self.entities
            .iter_with(&mut self.element_handles)
            .for_each(|(entity, _)| {
                to_kill.push(entity);
            });

        to_kill.into_iter().for_each(|entity| {
            self.delete_element(entity);
        });
    }
}

/// Handles user input comming from the editor and makes the required changes to the map.
fn handle_editor_input(player_inputs: Res<MatchInputs>, mut map_manager: MapManager) {
    for player in &player_inputs.players {
        if let Some(editor_input) = &player.editor_input {
            match editor_input {
                EditorInput::SpawnElement {
                    handle,
                    translation,
                    layer,
                } => {
                    map_manager.create_element(handle, translation, *layer as u32);
                }
                EditorInput::CreateLayer { id } => {
                    map_manager.create_layer(ustr(id));
                }
                EditorInput::DeleteLayer { layer } => {
                    map_manager.delete_layer(*layer as u32);
                }
                EditorInput::RenameLayer {
                    layer,
                    name: new_name,
                } => map_manager.rename_layer(*layer as u32, new_name),
                EditorInput::MoveEntity { entity, pos } => {
                    map_manager.move_element(*entity, pos);
                }
                EditorInput::DeleteEntity { entity } => {
                    map_manager.delete_element(*entity);
                }
                EditorInput::SetTilemap { layer, handle } => {
                    map_manager.set_layer_tilemap(*layer as u32, handle);
                }
                EditorInput::SetTile {
                    layer,
                    pos,
                    tilemap_tile_idx,
                    collision,
                } => {
                    map_manager.set_tile(*layer as u32, *pos, tilemap_tile_idx, *collision);
                }
                EditorInput::MoveLayer { layer, down } => {
                    map_manager.swap_layer(*layer as u32, *down)
                }
                EditorInput::RenameMap { name } => {
                    map_manager.rename_map(name);
                }
                EditorInput::RandomizeTiles {
                    tile_layers,
                    element_layers,
                    tile_size,
                } => {
                    let map_constructor = ShiftnanigansMapConstructor::new(
                        map_manager.get_size(),
                        *tile_size,
                        tile_layers,
                        element_layers,
                    );
                    map_constructor.construct_map(&mut map_manager);
                }
            }
        }
    }
}
