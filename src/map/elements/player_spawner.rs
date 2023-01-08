use crate::{metadata::BuiltinElementKind, networking::RollbackIdWrapper, utils::Sort};

use super::*;

pub struct PlayerSpawnerPlugin;
impl Plugin for PlayerSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentPlayerSpawner>()
            .add_rollback_system(RollbackStage::PreUpdate, pre_update_in_game)
            .extend_rollback_plugin(|plugin| {
                plugin
                    .register_rollback_type::<PlayerSpawner>()
                    .register_rollback_type::<CurrentPlayerSpawner>()
            });
    }
}

/// Marker component for player spawners
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct PlayerSpawner;

#[derive(Resource, Reflect, Default, Deref, DerefMut)]
#[reflect(Resource, Default)]
pub struct CurrentPlayerSpawner(pub usize);

fn pre_update_in_game(
    mut commands: Commands,
    player_inputs: Res<PlayerInputs>,
    players: Query<&PlayerIdx>,
    player_spawners: Query<(&Sort, &Transform), With<PlayerSpawner>>,
    non_hydrated_map_elements: Query<
        (Entity, &Sort, &Transform, &Handle<MapElementMeta>),
        Without<MapElementHydrated>,
    >,
    mut ridp: ResMut<RollbackIdWrapper>,
    mut current_spawner: ResMut<CurrentPlayerSpawner>,
    element_assets: Res<Assets<MapElementMeta>>,
) {
    let mut spawn_points = player_spawners.iter().collect::<Vec<_>>();
    // Hydrate any newly-spawned spawn points
    for (entity, sort, transform, map_element_handle) in &non_hydrated_map_elements {
        let map_element = element_assets.get(map_element_handle).unwrap();
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
            **current_spawner += 1;
            **current_spawner %= spawn_points.len().max(1);

            let Some((_, spawn_point)) = spawn_points.get(**current_spawner) else {
                break;
            };

            let mut spawn_location = spawn_point.translation;
            spawn_location.z += i as f32 * 0.1;

            commands.spawn((
                PlayerIdx(i),
                Transform::from_translation(spawn_location),
                Rollback::new(ridp.next_id()),
            ));
        }
    }
}
