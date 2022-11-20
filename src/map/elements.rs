use crate::{
    animation::AnimatedSprite,
    map::MapElementHydrated,
    metadata::MapElementMeta,
    physics::{collisions::CollisionWorld, KinematicBody},
    player::{input::PlayerInputs, PlayerIdx, MAX_PLAYERS},
    prelude::*,
};

pub struct MapElementsPlugin;

impl Plugin for MapElementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(player_spawner::PlayerSpawnerPlugin)
            .add_plugin(sproinger::SproingerPlugin);
    }
}

mod player_spawner {
    use crate::metadata::BuiltinElementKind;

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
        player_spawners: Query<&Transform, With<PlayerSpawner>>,
        non_hydrated_map_elements: Query<
            (Entity, &Transform, &MapElementMeta),
            Without<MapElementHydrated>,
        >,
        mut ridp: ResMut<RollbackIdProvider>,
    ) {
        let mut spawn_points = player_spawners.iter().collect::<Vec<_>>();
        // Hydrate any newly-spawned spawn points
        for (entity, transform, map_element) in &non_hydrated_map_elements {
            // TODO: Better way to tie the behavior to the map element?
            if matches!(map_element.builtin, BuiltinElementKind::PlayerSpawner) {
                commands
                    .entity(entity)
                    .insert(MapElementHydrated)
                    .insert(PlayerSpawner);
                spawn_points.push(transform);
            }
        }

        // For every player
        for i in 0..MAX_PLAYERS {
            let player = &player_inputs.players[i];

            // If the player is active, but not alive
            if player.active && !players.iter().any(|x| x.0 == i) {
                *current_spawner += 1;
                *current_spawner %= spawn_points.len().max(1);

                let Some(spawn_point) = spawn_points.get(*current_spawner) else {
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
}

mod sproinger {
    use crate::metadata::BuiltinElementKind;

    use super::*;

    const FORCE: f32 = 30.0;

    pub struct SproingerPlugin;
    impl Plugin for SproingerPlugin {
        fn build(&self, app: &mut App) {
            app.extend_rollback_schedule(|schedule| {
                schedule
                    .add_system_to_stage(RollbackStage::PreUpdateInGame, pre_update_in_game)
                    .add_system_to_stage(RollbackStage::UpdateInGame, update_in_game);
            })
            .extend_rollback_plugin(|plugin| plugin.register_rollback_type::<Sproinger>());
        }
    }

    #[derive(Component, Reflect, Default)]
    #[reflect(Component, Default)]
    pub struct Sproinger {
        frame: u32,
        sproinging: bool,
    }

    fn pre_update_in_game(
        mut commands: Commands,
        non_hydrated_map_elements: Query<(Entity, &MapElementMeta), Without<MapElementHydrated>>,
    ) {
        // Hydrate any newly-spawned sproingers
        for (entity, map_element) in &non_hydrated_map_elements {
            if let BuiltinElementKind::Sproinger { atlas_handle, .. } = &map_element.builtin {
                commands
                    .entity(entity)
                    .insert(MapElementHydrated)
                    .insert(Sproinger::default())
                    .insert(AnimatedSprite {
                        start: 0,
                        end: 6,
                        atlas: atlas_handle.inner.clone(),
                        repeat: false,
                        fps: 0.0,
                        ..default()
                    })
                    .insert(KinematicBody {
                        size: Vec2::new(32.0, 8.0),
                        offset: Vec2::new(0.0, -6.0),
                        has_mass: false,
                        ..default()
                    });
            }
        }
    }

    fn update_in_game(
        mut sproingers: Query<(Entity, &mut Sproinger, &mut AnimatedSprite)>,
        mut bodies: Query<&mut KinematicBody>,
        collision_world: CollisionWorld,
    ) {
        for (sproinger_ent, mut sproinger, mut sprite) in &mut sproingers {
            if sproinger.sproinging {
                match sproinger.frame {
                    1 => sprite.index = 2,
                    4 => sprite.index = 3,
                    8 => sprite.index = 4,
                    12 => sprite.index = 5,
                    20 => {
                        sprite.index = 0;
                        sproinger.sproinging = false;
                        sproinger.frame = 0;
                    }
                    _ => (),
                }
                sproinger.frame += 1;
            }

            for collider_ent in collision_world.actor_collisions(sproinger_ent) {
                let mut body = bodies.get_mut(collider_ent).unwrap();

                if !sproinger.sproinging {
                    body.velocity.y = FORCE;

                    sproinger.sproinging = true;
                }
            }
        }
    }
}
