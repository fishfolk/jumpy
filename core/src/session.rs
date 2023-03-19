use crate::prelude::*;

/// The implementation of a Jumpy game session.
///
/// This session allows you to do things like:
///
/// - Provide input
/// - Snapshot/Restore game state
/// - Access the session's ECS [`World`]
pub struct CoreSession {
    pub world: World,
    pub stages: SystemStages,
    pub scratch_world: Option<::bevy::ecs::world::World>,
    pub info: CoreSessionInfo,
}

/// Information needed to start a game session.
#[derive(Debug, Clone)]
pub struct CoreSessionInfo {
    /// The core metadata.
    pub meta: Arc<CoreMeta>,
    /// Metadata for the selected map.
    pub map_meta: MapMeta,
    /// The player selections.
    pub player_info: [Option<GameSessionPlayerInfo>; MAX_PLAYERS],
}

#[derive(Debug, Clone)]
pub struct GameSessionPlayerInfo {
    pub handle: Handle<PlayerMeta>,
    pub is_ai: bool,
}

impl CoreSession {
    /// Create a new game session
    pub fn new(mut info: CoreSessionInfo) -> Self {
        // Create session
        let mut session = Self {
            world: default(),
            stages: SystemStages::with_core_stages(),
            scratch_world: Some(::bevy::ecs::world::World::new()),
            info: info.clone(),
        };

        // Install modules
        crate::install_modules(&mut session);

        // Initialize systems
        for stage in &mut session.stages.stages {
            stage.initialize(&mut session.world);
        }

        // Initialize time resource
        session.world.init_resource::<Time>();
        // Initialize bevy world resource with an empty bevy world
        session.world.init_resource::<BevyWorld>();
        // Set the map
        session
            .world
            .insert_resource(LoadedMap(Arc::new(info.map_meta)));

        // Set player initial character selections
        let player_inputs = session.world.resource::<PlayerInputs>();
        let mut player_inputs = player_inputs.borrow_mut();
        for i in 0..MAX_PLAYERS {
            if let Some(info) = info.player_info[i].take() {
                player_inputs.players[i].active = true;
                player_inputs.players[i].selected_player = info.handle;
                player_inputs.players[i].is_ai = info.is_ai;
            }
        }

        session.set_metadata(info.meta);

        session
    }

    /// Set the game metadata.
    ///
    /// This may be used to change game metadata in the middle of the session.
    pub fn set_metadata(&mut self, metadata: Arc<CoreMeta>) {
        self.world.insert_resource(CoreMetaArc(metadata));
    }

    /// Provide a closure to update the game inputs.
    pub fn update_input<R, F: FnOnce(&mut PlayerInputs) -> R>(&mut self, update: F) -> R {
        let inputs = self.world.resource::<PlayerInputs>();
        let mut inputs = inputs.borrow_mut();

        update(&mut inputs)
    }

    pub fn restart(&mut self) {
        *self = Self::new(self.info.clone());
    }

    /// Run a single simulation frame
    pub fn advance(&mut self, bevy_world: &mut ::bevy::prelude::World) {
        puffin::profile_function!();

        // Update the window resource
        let window_resource = self.world.resource::<Window>();
        let bevy_windows = bevy_world.resource::<::bevy::window::Windows>();
        if let Some(window) = bevy_windows.get_primary() {
            window_resource.borrow_mut().size = Vec2::new(window.width(), window.height());
        }

        // Make bevy world available to the bones ECS world.
        {
            let world_resource = self.world.resource::<BevyWorld>();
            let mut world_resource = world_resource.borrow_mut();
            let mut scratch_world = self.scratch_world.take().unwrap();
            std::mem::swap(&mut scratch_world, bevy_world);
            world_resource.0 = Some(scratch_world);
        }
        for stage in &mut self.stages.stages {
            let stage_name = stage.name();
            puffin::profile_scope!("Run Stage", stage_name);
            stage.run(&mut self.world).unwrap();
        }

        // Advance the simulation time
        let time_resource = self.world.resource::<Time>();
        time_resource
            .borrow_mut()
            .advance_exact(std::time::Duration::from_secs_f32(1.0 / crate::FPS));

        self.world.maintain();

        // Swap the bevy world back to normal.
        {
            let world_resource = self.world.resource::<BevyWorld>();
            let mut world_resource = world_resource.borrow_mut();
            let mut scratch_world = world_resource.0.take().unwrap();
            std::mem::swap(bevy_world, &mut scratch_world);
            self.scratch_world = Some(scratch_world);
        }

        // Clear editor input from player inputs
        {
            let player_inputs = self.world.resource::<PlayerInputs>();
            let mut player_inputs = player_inputs.borrow_mut();
            for input in &mut player_inputs.players {
                input.editor_input = None;
            }
        }
    }

    /// Export the current map metadata by scanning the world entities. This means that the export
    /// will include any modifications to the map made at runtime ( most likely by the editor ).
    pub fn export_map(&self) -> MapMeta {
        let export_system =
            move |map_meta: Res<SpawnedMapMeta>,
                  entities: Res<Entities>,
                  tile_layers: Comp<TileLayer>,
                  spawned_map_layer_metas: Comp<SpawnedMapLayerMeta>,
                  tile_collisions: Comp<TileCollisionKind>,
                  tiles: Comp<Tile>,
                  transforms: Comp<Transform>,
                  element_handles: Comp<ElementHandle>| {
                let mut layers = map_meta
                    .layer_names
                    .iter()
                    .map(|name| MapLayerMeta {
                        id: name.clone(),
                        tilemap: default(),
                        tiles: default(),
                        elements: default(),
                    })
                    .collect::<Vec<_>>();

                // Export the tile layers
                for (_ent, (tile_layer, layer_meta)) in
                    entities.iter_with((&tile_layers, &spawned_map_layer_metas))
                {
                    let layer_idx = layer_meta.layer_idx;
                    let layer = &mut layers[layer_idx];
                    if tile_layer.atlas.path == AssetPath::default() {
                        // Skip layers with dummy atlases
                        continue;
                    }
                    layer.tilemap = Some(tile_layer.atlas.clone());
                    layer.tiles = tile_layer
                        .tiles
                        .iter()
                        .enumerate()
                        .filter_map(|(i, ent)| {
                            ent.map(|ent| {
                                let collision =
                                    tile_collisions.get(ent).copied().unwrap_or_default();
                                let tile = tiles.get(ent).unwrap();
                                let i = i as u32;
                                let y = i / map_meta.grid_size.x;
                                let x = i - (y * map_meta.grid_size.x);
                                MapTileMeta {
                                    pos: UVec2::new(x, y),
                                    idx: tile.idx as u32,
                                    collision,
                                }
                            })
                        })
                        .collect();
                }

                // Export the entity layers
                for (_ent, (element_handle, transform, layer_meta)) in
                    entities.iter_with((&element_handles, &transforms, &spawned_map_layer_metas))
                {
                    let layer_idx = layer_meta.layer_idx;
                    let layer = &mut layers[layer_idx];

                    layer.elements.push(ElementSpawn {
                        pos: transform.translation.truncate(),
                        element: element_handle.0.clone(),
                    });
                }

                // Return complete map metadata
                Ok(MapMeta {
                    name: map_meta.name.to_string(),
                    background: (*map_meta.background).clone(),
                    background_color: map_meta.background_color,
                    grid_size: map_meta.grid_size,
                    tile_size: map_meta.tile_size,
                    layers,
                })
            };

        self.world.run_initialized_system(export_system).unwrap()
    }

    /// Snapshot the world state
    pub fn snapshot(&self) -> World {
        self.world.clone()
    }

    /// Restore the world state
    ///
    /// Will write the current state to `world`.
    pub fn restore(&mut self, world: &mut World) {
        std::mem::swap(&mut self.world, world)
    }
}
