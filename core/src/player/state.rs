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

/// Helper macro that adds the `player_state_transition` and `handle_player_state` systems from
/// `module` to the appropriate stages.
macro_rules! add_state_module {
    ($session:ident, $module:ident) => {
        $session
            .stages
            .add_system_to_stage(PlayerStateStage, $module::player_state_transition);
        $session
            .stages
            .add_system_to_stage(CoreStage::PreUpdate, $module::handle_player_state);
    };
}

pub fn install(session: &mut GameSession) {
    // Add the player state stage
    session
        .stages
        .insert_stage_before(CoreStage::PreUpdate, PlayerStateStageImpl::new());

    session
        .stages
        .add_system_to_stage(CoreStage::Last, update_player_state_age);

    add_state_module!(session, default);
    add_state_module!(session, idle);
    add_state_module!(session, crouch);
    add_state_module!(session, midair);
    add_state_module!(session, walk);
    add_state_module!(session, dead);
}

fn update_player_state_age(entities: Res<Entities>, mut player_states: CompMut<PlayerState>) {
    for (_ent, state) in entities.iter_with(&mut player_states) {
        state.age = state.age.saturating_add(1);
    }
}

fn use_drop_or_grab_items(
    player_ent: Entity,
    meta: &PlayerMeta,
    control: &PlayerControl,
    inventory: &Inventory,
    collision_world: &CollisionWorld,
    items: &Comp<Item>,
    held_items: &[Entity],
    player_events: &mut PlayerEvents,
    audio_events: &mut AudioEvents,
) {
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
                // Filter out any items held by other players
                .filter(|ent| !held_items.contains(ent))
                .collect::<Vec<_>>();

            // Grab the first item we are touching
            if let Some(item) = colliders.get(0) {
                // Add the item to the player inventory
                player_events.set_inventory(player_ent, Some(*item));

                // Play grab sound
                audio_events.play(meta.sounds.grab.clone(), meta.sounds.grab_volume);
            }

        // If we are already carrying an item
        } else {
            // Drop it
            player_events.set_inventory(player_ent, None);

            // Play drop sound
            audio_events.play(meta.sounds.drop.clone(), meta.sounds.drop_volume);
        }
    }

    // If we are using an item
    if control.shoot_just_pressed && inventory.is_some() {
        player_events.use_item(player_ent);
    }
}
