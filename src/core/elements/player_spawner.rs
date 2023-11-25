use crate::prelude::*;

/// Player spawner element
#[derive(HasSchema, Default, Debug, Clone, Copy)]
#[type_data(metadata_asset("player_spawner"))]
#[repr(C)]
pub struct PlayerSpawnerMeta;

pub fn game_plugin(game: &mut Game) {
    PlayerSpawnerMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::First, hydrate)
        .add_system_to_stage(CoreStage::First, update);
}

/// Marker component for player spawners.
#[derive(Clone, Debug, HasSchema, Default)]
#[type_data(metadata_asset("player_spawner"))]
#[repr(C)]
pub struct PlayerSpawner;

/// Resource that stores the next spawner to use when spawning a player.
#[derive(Clone, Debug, HasSchema, Default)]
pub struct CurrentSpawner(pub usize);

fn hydrate(
    entities: Res<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,
    mut player_spawners: CompMut<PlayerSpawner>,
    mut spawner_manager: SpawnerManager,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    for entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let element_handle = element_handles.get(entity).unwrap();
        let element_meta = assets.get(element_handle.0);

        if let Ok(PlayerSpawner) = assets.get(element_meta.data).try_cast_ref() {
            hydrated.insert(entity, MapElementHydrated);
            player_spawners.insert(entity, PlayerSpawner);

            spawner_manager.create_grouped_spawner(entity, vec![], &player_spawners, &entities);
        }
    }
}

fn update(
    mut entities: ResMutInit<Entities>,
    mut current_spawner: ResMutInit<CurrentSpawner>,
    player_spawners: Comp<PlayerSpawner>,
    mut player_indexes: CompMut<PlayerIdx>,
    mut transforms: CompMut<Transform>,
    player_inputs: Res<MatchInputs>,
    mut spawner_manager: SpawnerManager,
) {
    let alive_players = entities
        .iter_with(&player_indexes)
        .map(|(_ent, pidx)| pidx.0)
        .collect::<Vec<_>>();
    let spawn_points = entities
        .iter_with((&player_spawners, &transforms))
        .map(|(_ent, (_spawner, transform))| transform.translation)
        .collect::<Vec<_>>();

    // For every player
    for i in 0..MAX_PLAYERS as u32 {
        let player = &player_inputs.players[i as usize];

        // If the player is active, but not alive
        if player.active && !alive_players.contains(&i) {
            // Increment the spawner index
            current_spawner.0 += 1;
            current_spawner.0 %= spawn_points.len().max(1);

            let Some(mut spawn_point) = spawn_points.get(current_spawner.0).copied() else {
                return;
            };

            // Make sure each player spawns at a different z level ( give enough room for 10 players
            // to fit between map layers )
            spawn_point.z += i as f32 * MAP_LAYERS_GAP_DEPTH / 10.0;

            let player_ent = entities.create();
            player_indexes.insert(player_ent, PlayerIdx(i));
            transforms.insert(player_ent, Transform::from_translation(spawn_point));

            spawner_manager.insert_spawned_entity_into_grouped_spawner(
                player_ent,
                &player_spawners,
                &entities,
            );
        }
    }
}
