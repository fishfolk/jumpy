use crate::{metadata::BuiltinElementKind, utils::Sort};

use super::*;

pub struct PlayerSpawnerPlugin;
impl Plugin for PlayerSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.extend_rollback_schedule(|schedule| {
            schedule.add_system_to_stage(RollbackStage::PreUpdateInGame, pre_update_in_game);
        })
        .extend_rollback_plugin(|plugin| plugin.register_rollback_type::<PlayerSpawner>());
    }
}

/// Marker component for player spawners
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct PlayerSpawner;

fn pre_update_in_game(
    mut current_spawner: Local<usize>,
    mut commands: Commands,
    player_inputs: Res<PlayerInputs>,
    players: Query<&PlayerIdx>,
    player_spawners: Query<(&Sort, &Transform), With<PlayerSpawner>>,
    non_hydrated_map_elements: Query<
        (Entity, &Sort, &Transform, &MapElementMeta),
        Without<MapElementHydrated>,
    >,
    mut ridp: ResMut<RollbackIdProvider>,
) {
    let mut spawn_points = player_spawners.iter().collect::<Vec<_>>();
    // Hydrate any newly-spawned spawn points
    for (entity, sort, transform, map_element) in &non_hydrated_map_elements {
        // TODO: Better way to tie the behavior to the map element?
        if matches!(map_element.builtin, BuiltinElementKind::PlayerSpawner) {
            commands
                .entity(entity)
                .insert(MapElementHydrated)
                .insert(PlayerSpawner);
            spawn_points.push((sort, transform));
        }
    }
    spawn_points.sort_by_key(|x| x.0);

    // For every player
    for i in 0..MAX_PLAYERS {
        let player = &player_inputs.players[i];

        // If the player is active, but not alive
        if player.active && !players.iter().any(|x| x.0 == i) {
            *current_spawner += 1;
            *current_spawner %= spawn_points.len().max(1);

            let Some((_, spawn_point)) = spawn_points.get(*current_spawner) else {
                break;
            };

            commands
                .spawn()
                .insert(PlayerIdx(i))
                .insert(**spawn_point)
                .insert(Rollback::new(ridp.next_id()));
        }
    }
}
