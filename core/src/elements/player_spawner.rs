use crate::{prelude::*, MAX_PLAYERS};

pub fn install(session: &mut CoreSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::First, hydrate)
        .add_system_to_stage(CoreStage::First, update);
}

/// Marker component for player spawners.
#[derive(Clone, Debug, TypeUlid)]
#[ulid = "01GP4YT7VARRVHFWJ46HNSN09P"]
pub struct PlayerSpawner;

/// Resource that stores the next spawner to use when spawning a player.
#[derive(Clone, Debug, TypeUlid, Default)]
#[ulid = "01GP4YVEQGVQATG3KSPC0SD37N"]
pub struct CurrentSpawner(pub usize);

fn hydrate(
    entities: Res<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut player_spawners: CompMut<PlayerSpawner>,
    mut spawner_manager: SpawnerManager,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    for entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let element_handle = element_handles.get(entity).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        if let BuiltinElementKind::PlayerSpawner = &element_meta.builtin {
            hydrated.insert(entity, MapElementHydrated);
            player_spawners.insert(entity, PlayerSpawner);

            spawner_manager.insert_grouped_spawner(entity, vec![], &player_spawners);
        }
    }
}

fn update(
    game_meta: Res<CoreMetaArc>,
    mut entities: ResMut<Entities>,
    mut current_spawner: ResMut<CurrentSpawner>,
    player_spawners: Comp<PlayerSpawner>,
    mut player_indexes: CompMut<PlayerIdx>,
    mut transforms: CompMut<Transform>,
    player_inputs: Res<PlayerInputs>,
    mut invincibles: CompMut<Invincibility>,
    mut spawner_manager: SpawnerManager,
    mut element_kill_callbacks: CompMut<ElementKillCallback>,
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
    for i in 0..MAX_PLAYERS {
        let player = &player_inputs.players[i];

        // If the player is active, but not alive
        if player.active && !alive_players.contains(&i) {
            // Increment the spawner index
            current_spawner.0 += 1;
            current_spawner.0 %= spawn_points.len().max(1);

            let Some(mut spawn_point) = spawn_points.get(current_spawner.0).copied() else { return };

            // Make sure each player spawns at a different z level ( give enough room for 10 players
            // to fit between map layers )
            spawn_point.z += i as f32 * MAP_LAYERS_GAP_DEPTH / 10.0;

            let player_ent = entities.create();
            player_indexes.insert(player_ent, PlayerIdx(i));
            transforms.insert(player_ent, Transform::from_translation(spawn_point));
            invincibles.insert(
                player_ent,
                Invincibility::new(game_meta.config.respawn_invincibility_time),
            );

            element_kill_callbacks.insert(
                player_ent,
                ElementKillCallback::new(player_kill_callback(player_ent)),
            );

            spawner_manager.insert_spawned_entity_into_grouped_spawner(player_ent, &player_spawners);
        }
    }
}

fn player_kill_callback(player_entity: Entity) -> System {
    (move |mut entities: ResMut<Entities>,
           attachments: Comp<Attachment>,
           player_layers: Comp<PlayerLayers>| {
        entities
            .iter_with(&attachments)
            .filter(|(_, attachment)| attachment.entity == player_entity)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>()
            .iter()
            .for_each(|entity| {
                entities.kill(*entity);
            });
        let layers = player_layers.get(player_entity).unwrap();
        entities.kill(layers.fin_ent);
        entities.kill(layers.face_ent);
        entities.kill(player_entity);
    })
    .system()
}
