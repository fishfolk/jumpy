use super::*;

pub use stage::*;
mod stage;

pub use states::*;
mod states;

/// The state of the player controller.
#[derive(Clone, HasSchema, Default)]
pub struct PlayerState {
    /// The ID for the current state.
    pub current: Ustr,
    /// The number of frames that this state has been active.
    pub age: u64,
    /// The ID of the state that the player was in in the last frame.
    pub last: Ustr,
}
impl PlayerState {
    /// Adds a system to the appropriate stage in a [`Session`] for a player state transition.
    pub fn add_player_state_transition_system<Args>(
        session: &mut Session,
        system: impl IntoSystem<Args, (), (), Sys = StaticSystem<(), ()>>,
    ) {
        session.stages.add_system_to_stage(PlayerStateStage, system);
    }

    /// Adds a system to the appropriate stage in a [`Session`] for a player state update.
    pub fn add_player_state_update_system<Args>(
        session: &mut Session,
        system: impl IntoSystem<Args, (), (), Sys = StaticSystem<(), ()>>,
    ) {
        session
            .stages
            .add_system_to_stage(CoreStage::PreUpdate, system);
    }
}

pub fn plugin(session: &mut Session) {
    // Add the player state stage
    session
        .stages
        .insert_stage_before(CoreStage::PreUpdate, PlayerStateStageImpl::new());

    session
        .stages
        .add_system_to_stage(CoreStage::Last, update_player_state_age);

    crouch::install(session);
    dead::install(session);
    default::install(session);
    drive_jellyfish::install(session);
    idle::install(session);
    incapacitated::install(session);
    midair::install(session);
    walk::install(session);
}

fn update_player_state_age(entities: Res<Entities>, mut player_states: CompMut<PlayerState>) {
    for (_ent, state) in entities.iter_with(&mut player_states) {
        state.age = state.age.saturating_add(1);
    }
}

fn use_drop_or_grab_items_system(id: Ustr) -> StaticSystem<(), ()> {
    (move |entities: Res<Entities>,
           player_inputs: Res<MatchInputs>,
           player_indexes: Comp<PlayerIdx>,
           player_states: Comp<PlayerState>,
           assets: Res<AssetServer>,
           items: Comp<Item>,
           collision_world: CollisionWorld,
           mut inventories: CompMut<Inventory>,
           mut audio_center: ResMut<AudioCenter>,
           mut commands: Commands| {
        // Collect a list of items that are being held by players
        let held_items = entities
            .iter_with(&inventories)
            .filter_map(|(_ent, inventory)| inventory.0)
            .collect::<Vec<_>>();

        for (player_ent, (player_state, player_idx, inventory)) in
            entities.iter_with((&player_states, &player_indexes, &mut inventories))
        {
            if player_state.current != id {
                continue;
            }
            let meta_handle = player_inputs.players[player_idx.0 as usize].selected_player;
            let meta = assets.get(meta_handle);

            let control = &player_inputs.players[player_idx.0 as usize].control;
            // If we are grabbing
            if control.grab_just_pressed {
                if inventory.is_none() {
                    // If we don't have an item
                    let colliders = collision_world
                        // Get all things colliding with the player
                        .actor_collisions(player_ent)
                        .into_iter()
                        // Filter out anything not an item
                        .filter(|ent| items.contains(*ent))
                        // TODO: Use the ItemGrabbed tag for this detection after fixing the ItemGrabbed handling
                        // Filter out any items held by other players
                        .filter(|ent| !held_items.contains(ent))
                        .collect::<Vec<_>>();

                    // Grab the first item we are touching
                    if let Some(item) = colliders.get(0) {
                        // Add the item to the player inventory
                        commands.add(PlayerCommand::set_inventory(player_ent, Some(*item)));

                        // Play grab sound
                        audio_center.play_sound(meta.sounds.grab, meta.sounds.grab_volume);
                    }

                // If we are already carrying an item
                } else {
                    // Drop it
                    commands.add(PlayerCommand::set_inventory(player_ent, None));

                    // Play drop sound
                    audio_center.play_sound(meta.sounds.drop, meta.sounds.drop_volume);
                }
            }

            // If we are using an item
            if control.shoot_just_pressed && inventory.is_some() {
                commands.add(PlayerCommand::use_item(player_ent));
            }
        }
    })
    .system()
}
