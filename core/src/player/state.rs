use super::*;

pub use stage::*;
mod stage;

use states::*;
mod states;

/// The state of the player controller.
#[derive(Clone, TypeUlid, Default)]
#[ulid = "01GP4E4BH47RN41QS66QBD679Q"]
pub struct PlayerState {
    /// The ID for the current state.
    pub current: Key,
    /// The number of frames that this state has been active.
    pub age: u64,
    /// The ID of the state that the player was in in the last frame.
    pub last: Key,
}
impl PlayerState {
    /// Adds a system to the appropriate stage in a [`GameSession`] for a player state transition.
    pub fn add_player_state_transition_system<Args>(
        session: &mut CoreSession,
        system: impl IntoSystem<Args, ()>,
    ) {
        session.stages.add_system_to_stage(PlayerStateStage, system);
    }

    /// Adds a system to the appropriate stage in a [`GameSession`] for a player state update.
    pub fn add_player_state_update_system<Args>(
        session: &mut CoreSession,
        system: impl IntoSystem<Args, ()>,
    ) {
        session
            .stages
            .add_system_to_stage(CoreStage::PreUpdate, system);
    }
}

pub fn install(session: &mut CoreSession) {
    // Add the player state stage
    session
        .stages
        .insert_stage_before(CoreStage::PreUpdate, PlayerStateStageImpl::new());

    session
        .stages
        .add_system_to_stage(CoreStage::Last, update_player_state_age);

    default::install(session);
    idle::install(session);
    crouch::install(session);
    midair::install(session);
    walk::install(session);
    dead::install(session);
    incapacitated::install(session);
}

fn update_player_state_age(entities: Res<Entities>, mut player_states: CompMut<PlayerState>) {
    for (_ent, state) in entities.iter_with(&mut player_states) {
        state.age = state.age.saturating_add(1);
    }
}

fn use_drop_or_grab_items_system(id: Key) -> System {
    (move |entities: Res<Entities>,
           player_inputs: Res<PlayerInputs>,
           player_indexes: Comp<PlayerIdx>,
           player_states: Comp<PlayerState>,
           player_assets: BevyAssets<PlayerMeta>,
           items: Comp<Item>,
           collision_world: CollisionWorld,
           mut inventories: CompMut<Inventory>,
           mut audio_events: ResMut<AudioEvents>,
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
            let meta_handle = player_inputs.players[player_idx.0]
                .selected_player
                .get_bevy_handle();
            let Some(meta) = player_assets.get(&meta_handle) else { continue; };

            let control = &player_inputs.players[player_idx.0].control;
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
                        audio_events.play(meta.sounds.grab.clone(), meta.sounds.grab_volume);
                    }

                // If we are already carrying an item
                } else {
                    // Drop it
                    commands.add(PlayerCommand::set_inventory(player_ent, None));

                    // Play drop sound
                    audio_events.play(meta.sounds.drop.clone(), meta.sounds.drop_volume);
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
