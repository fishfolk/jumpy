use super::*;

pub static ID: Lazy<Ustr> = Lazy::new(|| ustr("core::incapacitated"));

pub fn install(session: &mut Session) {
    PlayerState::add_player_state_update_system(session, handle_player_state);
}

const SLOWING_SPEED: f32 = 0.3;

pub fn handle_player_state(
    entities: Res<Entities>,
    mut player_states: CompMut<PlayerState>,
    player_indexes: Comp<PlayerIdx>,
    assets: Res<AssetServer>,
    player_inputs: Res<MatchInputs>,
    atlas_sprites: Comp<AtlasSprite>,
    mut animations: CompMut<AnimationBankSprite>,
    mut bodies: CompMut<KinematicBody>,
) {
    for (player_ent, (state, animation, body, player_idx, atlas_sprite)) in entities.iter_with((
        &mut player_states,
        &mut animations,
        &mut bodies,
        &player_indexes,
        &atlas_sprites,
    )) {
        if state.current != *ID {
            continue;
        };

        let meta_handle = player_inputs.players[player_idx.0 as usize].selected_player;
        let meta = assets.get(meta_handle);

        match state.age {
            0 => {
                // TODO find right animation
                animation.current = "rise".into();
                PlayerCommand::set_inventory(player_ent, None);

                if body.velocity.x.abs() < meta.stats.walk_speed {
                    body.velocity.x = 5. * if atlas_sprite.flip_x { -1.0f32 } else { 1.0 };
                }
            }
            n if n < 80 => {
                if body.velocity.x.abs() < SLOWING_SPEED {
                    body.velocity.x = 0.;
                } else {
                    body.velocity.x -= body.velocity.x.signum() * SLOWING_SPEED
                }
            }
            n if n >= 80 => {
                state.current = *idle::ID;
                animation.current = ustr("idle");
            }
            _ => (),
        }
    }
}
