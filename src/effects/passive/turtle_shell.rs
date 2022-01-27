use hecs::{Entity, World};

use crate::player::{PlayerEventQueue, PlayerState};
use crate::PlayerEvent;

pub const EFFECT_FUNCTION_ID: &str = "turtle_shell";

pub fn effect_function(
    world: &mut World,
    player_entity: Entity,
    _item_entity: Option<Entity>,
    event: PlayerEvent,
) {
    if let PlayerEvent::ReceiveDamage { is_from_left, .. } = event {
        let state = world.get::<PlayerState>(player_entity).unwrap();
        let mut events = world.get_mut::<PlayerEventQueue>(player_entity).unwrap();

        if state.is_facing_left != is_from_left {
            events
                .queue
                .push(PlayerEvent::DamageBlocked { is_from_left });
        }
    }
}
