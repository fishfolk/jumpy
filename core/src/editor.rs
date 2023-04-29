use crate::{map::z_depth_for_map_layer, prelude::*};

pub fn install(session: &mut CoreSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, handle_editor_input);
}

/// Macro to "derive" ( not really a derive macro ) SystemParam for a struct.
macro_rules! impl_system_param {
    (
        pub struct $t:ident<'a> {
            $(
                $( #[$attrs:meta] )*
                $f_name:ident: $f_ty:ty
            ),*
            $(,)?
        }
    ) => {
        pub struct $t<'a> {
            $(
                $( #[$attrs] )*
                pub $f_name: $f_ty
            ),*
        }

        impl<'a> SystemParam for $t<'a> {
            type State = (
                $(
                    <$f_ty as SystemParam>::State
                ),*
            );
            type Param<'p> = $t<'p>;

            fn initialize(world: &mut World) {
                $(
                    <$f_ty as SystemParam>::initialize(world);
                )*
            }

            fn get_state(world: &World) -> Self::State {
                (
                    $(
                        <$f_ty as SystemParam>::get_state(world)
                    ),*
                )
            }

            fn borrow(state: &mut Self::State) -> Self::Param<'_> {
                let (
                    $(
                        $f_name
                    ),*
                ) = state;
                let (
                    $(
                        $f_name
                    ),*
                ) = (
                    $(
                        <$f_ty as SystemParam>::borrow($f_name)
                    ),*
                );

                Self::Param {
                    $(
                        $f_name
                    ),*
                }
            }
        }
    };
}

impl_system_param! {
    pub struct MapInterface<'a> {
        commands: Commands<'a>,
        entities: ResMut<'a, Entities>,
        spawned_map_meta: ResMut<'a, SpawnedMapMeta>,
        element_handles: CompMut<'a, ElementHandle>,
        transforms: CompMut<'a, Transform>,
        spawned_map_layer_metas: CompMut<'a, SpawnedMapLayerMeta>,
        tile_layers: CompMut<'a, TileLayer>,
        tiles: CompMut<'a, Tile>,
        tile_collisions: CompMut<'a, TileCollisionKind>
    }
}

impl<'a> MapInterface<'a> {
    fn create_element(
        &mut self,
        element_meta_handle: &Handle<ElementMeta>,
        translation: &Vec2,
        layer_index: usize,
    ) {
        let entity = self.entities.create();
        self.element_handles
            .insert(entity, ElementHandle(element_meta_handle.clone()));
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
    fn create_layer(&mut self, name: String) {
        let entity = self.entities.create();
        let layer_index = self.spawned_map_meta.layer_names.len();
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
    fn delete_layer(&mut self, layer_index: usize) {
        let layer_count = self.spawned_map_meta.layer_names.len();
        let layers_to_decrement = layer_count - layer_index;
        self.spawned_map_meta.layer_names = self
            .spawned_map_meta
            .layer_names
            .clone()
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(i, name)| if i == layer_index { None } else { Some(name) })
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
    fn rename_layer(&mut self, layer_index: usize, name: &str) {
        self.spawned_map_meta.layer_names = self
            .spawned_map_meta
            .layer_names
            .clone()
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, n)| if i == layer_index { name.to_owned() } else { n })
            .collect();
    }
    fn move_element(&mut self, entity: Entity, position: &Vec2) {
        let transform = self.transforms.get_mut(entity).unwrap();
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
    fn delete_element(&mut self, entity: Entity) {
        self.entities.kill(entity);
    }
    fn set_layer_tilemap(&mut self, layer_index: usize, tilemap: &Option<Handle<Atlas>>) {
        if let Some((_, (tile_layer, _))) = self
            .entities
            .iter_with((&mut self.tile_layers, &self.spawned_map_layer_metas))
            .find(|x| x.1 .1.layer_idx == layer_index)
        {
            if let Some(handle) = tilemap {
                tile_layer.atlas = handle.clone();
            } else {
                tile_layer.atlas = default();
            }
        };
    }
    fn set_tile(
        &mut self,
        layer_index: usize,
        position: UVec2,
        tilemap_tile_index: &Option<usize>,
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
    fn swap_layer(&mut self, layer_index: usize, is_downward: bool) {
        let origin_layer_index = layer_index;
        let other_layer_index = if is_downward {
            origin_layer_index + 1
        } else {
            origin_layer_index - 1
        };
        let mut layer_names = self.spawned_map_meta.layer_names.to_vec();
        layer_names.swap(origin_layer_index, other_layer_index);
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
    fn rename_map(&mut self, name: String) {
        self.spawned_map_meta.name = name.into();
    }
}

fn handle_editor_input(player_inputs: Res<PlayerInputs>, mut map_interface: MapInterface) {
    for player in &player_inputs.players {
        if let Some(editor_input) = &player.editor_input {
            match editor_input {
                EditorInput::SpawnElement {
                    handle,
                    translation,
                    layer,
                } => {
                    map_interface.create_element(handle, translation, *layer as usize);
                }
                EditorInput::CreateLayer { id } => {
                    map_interface.create_layer(id.clone());
                }
                EditorInput::DeleteLayer { layer } => {
                    map_interface.delete_layer(*layer as usize);
                }
                EditorInput::RenameLayer {
                    layer,
                    name: new_name,
                } => map_interface.rename_layer(*layer as usize, new_name),
                EditorInput::MoveEntity { entity, pos } => {
                    map_interface.move_element(*entity, pos);
                }
                EditorInput::DeleteEntity { entity } => {
                    map_interface.delete_element(*entity);
                }
                EditorInput::SetTilemap { layer, handle } => {
                    map_interface.set_layer_tilemap(*layer as usize, handle);
                }
                EditorInput::SetTile {
                    layer,
                    pos,
                    tilemap_tile_idx,
                    collision,
                } => {
                    map_interface.set_tile(*layer as usize, *pos, tilemap_tile_idx, *collision);
                }
                EditorInput::MoveLayer { layer, down } => {
                    map_interface.swap_layer(*layer as usize, *down)
                }
                EditorInput::RenameMap { name } => {
                    map_interface.rename_map(name.clone());
                }
            }
        }
    }
}
