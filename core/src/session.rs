use crate::prelude::*;

/// The implementation of a Jumpy game session.
///
/// This session allows you to do things like:
///
/// - Provide input
/// - Snapshot/Restore game state
/// - Access the session's ECS [`World`]
pub struct GameSession {
    pub world: World,
    pub stages: SystemStages,
    pub scratch_world: Option<::bevy::ecs::world::World>,
    pub info: GameSessionInfo,
}

/// Information needed to start a game session.
#[derive(Debug, Clone)]
pub struct GameSessionInfo {
    /// The core metadata.
    pub meta: Arc<CoreMeta>,
    /// The selected map
    pub map: Handle<MapMeta>,
    /// The player selections.
    pub player_info: [Option<Handle<PlayerMeta>>; MAX_PLAYERS],
}

impl GameSession {
    /// Create a new game session
    pub fn new(mut info: GameSessionInfo) -> Self {
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
        session.world.resources.init::<Time>();
        // Initialize bevy world resource with an empty bevy world
        session.world.resources.init::<BevyWorld>();
        // Set the map
        session.world.resources.insert(MapHandle(info.map));

        // Set player initial character selections
        let player_inputs = session.world.resources.get::<PlayerInputs>();
        let mut player_inputs = player_inputs.borrow_mut();
        for i in 0..MAX_PLAYERS {
            if let Some(player) = info.player_info[i].take() {
                player_inputs.players[i].active = true;
                player_inputs.players[i].selected_player = player;
            }
        }

        session.set_metadata(info.meta);

        session
    }

    /// Set the game metadata.
    ///
    /// This may be used to change game metadata in the middle of the session.
    pub fn set_metadata(&mut self, metadata: Arc<CoreMeta>) {
        self.world.resources.insert(CoreMetaArc(metadata));
    }

    /// Provide a closure to update the game inputs.
    pub fn update_input<R, F: FnOnce(&mut PlayerInputs) -> R>(&mut self, update: F) -> R {
        let inputs = self.world.resources.get::<PlayerInputs>();
        let mut inputs = inputs.borrow_mut();

        update(&mut inputs)
    }

    pub fn restart(&mut self) {
        *self = Self::new(self.info.clone());
    }

    /// Run a single simulation frame
    pub fn advance(&mut self, bevy_world: &mut ::bevy::prelude::World) {
        // Update the window resource
        let window_resource = self.world.resources.get::<Window>();
        let bevy_windows = bevy_world.resource::<::bevy::window::Windows>();
        if let Some(window) = bevy_windows.get_primary() {
            window_resource.borrow_mut().size = Vec2::new(window.width(), window.height());
        }

        // Make bevy world available to the bones ECS world.
        {
            let world_resource = self.world.resources.get::<BevyWorld>();
            let mut world_resource = world_resource.borrow_mut();
            let mut scratch_world = self.scratch_world.take().unwrap();
            std::mem::swap(&mut scratch_world, bevy_world);
            world_resource.0 = Some(scratch_world);
        }
        for stage in &mut self.stages.stages {
            stage.run(&mut self.world).unwrap();
        }

        // Advance the simulation time
        let time_resource = self.world.resources.get::<Time>();
        time_resource.borrow_mut().elapsed += 1.0 / crate::FPS;

        self.world.maintain();

        // Swap the bevy world back to normal.
        {
            let world_resource = self.world.resources.get::<BevyWorld>();
            let mut world_resource = world_resource.borrow_mut();
            let mut scratch_world = world_resource.0.take().unwrap();
            std::mem::swap(bevy_world, &mut scratch_world);
            self.scratch_world = Some(scratch_world);
        }
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
