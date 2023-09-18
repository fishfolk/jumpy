use super::*;

pub static ID: Lazy<Ustr> = Lazy::new(|| ustr("core::dead"));

pub fn install(session: &mut Session) {
    PlayerState::add_player_state_transition_system(session, player_state_transition);
    PlayerState::add_player_state_update_system(session, handle_player_state);
}

pub fn player_state_transition(
    entities: Res<Entities>,
    killed_players: Comp<PlayerKilled>,
    mut player_states: CompMut<PlayerState>,
) {
    for (_ent, (state, _killed)) in entities.iter_with((&mut player_states, &killed_players)) {
        state.current = *ID;
    }
}

pub fn handle_player_state(
    entities: Res<Entities>,
    mut commands: Commands,
    player_states: Comp<PlayerState>,
    killed_players: Comp<PlayerKilled>,
    sprites: Comp<AtlasSprite>,
    transform: Comp<Transform>,
    player_layers: Comp<PlayerLayers>,
    mut player_body_attachments: CompMut<PlayerBodyAttachment>,
    mut kinematic_bodies: CompMut<KinematicBody>,
    mut animations: CompMut<AnimationBankSprite>,
) {
    for (player_ent, (state, animation, killed_player)) in
        entities.iter_with((&player_states, &mut animations, &killed_players))
    {
        if state.current != *ID {
            continue;
        };

        if state.age == 0 {
            let sprite = sprites.get(player_ent).unwrap();
            let player_on_right = !sprite.flip_x;
            let transform = transform.get(player_ent).unwrap();
            let layers = player_layers.get(player_ent).unwrap();

            // Knock the player's hat off if they had one.
            if let Some(hat_ent) = layers.hat_ent {
                player_body_attachments.remove(hat_ent);
                kinematic_bodies.get_mut(hat_ent).unwrap().is_deactivated = false;
            }

            animation.current = match killed_player.hit_from {
                Some(hit_from)
                    if {
                        let is_hit_right = transform.translation.x < hit_from.x;
                        (player_on_right && is_hit_right) || (!player_on_right && !is_hit_right)
                    } =>
                {
                    "death_spine".into()
                }
                _ => "death_belly".into(),
            };
        }

        if state.age >= 80 {
            commands.add(PlayerCommand::despawn(player_ent));
        }
    }
}
