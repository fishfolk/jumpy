use super::*;

#[derive(Debug)]
pub struct PlayerStateStage;

impl StageLabel for PlayerStateStage {
    fn name(&self) -> String {
        format!("{self:?}")
    }

    fn id(&self) -> Ulid {
        PlayerStateStageImpl::ID
    }
}

#[derive(Default)]
pub struct PlayerStateStageImpl {
    systems: Vec<StaticSystem<(), ()>>,
}

impl PlayerStateStageImpl {
    pub fn new() -> Self {
        default()
    }
}

impl PlayerStateStageImpl {
    pub const ID: Ulid = Ulid(2022686805174362721866480948664103805);
}

impl SystemStage for PlayerStateStageImpl {
    fn id(&self) -> Ulid {
        Self::ID
    }

    fn name(&self) -> String {
        "PlayerStateStage".into()
    }

    fn run(&mut self, world: &World) {
        trace!("Starting player state transitions");
        loop {
            // Get the current player states
            let last_player_states = world.run_system(
                |entities: Res<Entities>,
                 player_indexes: Comp<PlayerIdx>,
                 player_states: Comp<PlayerState>| {
                    let mut states: [Option<Ustr>; MAX_PLAYERS] = std::array::from_fn(|_| None);
                    for (_ent, (idx, state)) in
                        entities.iter_with((&player_indexes, &player_states))
                    {
                        states[idx.0 as usize] = Some(state.current);
                    }

                    states
                },
                (),
            );
            trace!(?last_player_states, "Checcking current states");

            trace!("Running state transitions");
            // Run all of the player state systems
            for system in &mut self.systems {
                system.run(world, ());
            }

            // Get whether the states have changed
            let has_changed = world.run_system(
                move |entities: Res<Entities>,
                      player_indexes: Comp<PlayerIdx>,
                      mut player_states: CompMut<PlayerState>| {
                    let mut has_changed = false;
                    for (_ent, (idx, state)) in
                        entities.iter_with((&player_indexes, &mut player_states))
                    {
                        let old_state = last_player_states[idx.0 as usize].unwrap();

                        if old_state != state.current {
                            state.last = old_state;
                            state.age = 0;
                            has_changed = true;
                        }
                    }

                    has_changed
                },
                (),
            );

            // If the states haven't changed
            if !has_changed {
                trace!("No state changes, done with state transition loop");
                // Then we are finished applying player state transitions
                break;
            }
        }
    }

    fn add_system(&mut self, system: StaticSystem<(), ()>) {
        self.systems.push(system.system())
    }
}
