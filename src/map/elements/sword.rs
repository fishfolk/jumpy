use crate::player::PlayerKillCommand;

use super::*;

pub struct SwordPlugin;
impl Plugin for SwordPlugin {
    fn build(&self, app: &mut App) {
        app.add_rollback_system(RollbackStage::PreUpdate, pre_update_in_game)
            .add_rollback_system(RollbackStage::Update, update_in_game)
            .extend_rollback_plugin(|plugin| plugin.register_rollback_type::<Sword>());
    }
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component, Default)]
pub struct Sword {
    pub state: SwordState,
    dropped_time: f32,
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component, Default)]
pub enum SwordState {
    #[default]
    Idle,
    Swinging {
        frame: usize,
    },
    Cooldown {
        frame: usize,
    },
}

const COOLDOWN_FRAMES: usize = 13;
const ATTACK_FPS: f32 = 10.0;

fn pre_update_in_game(
    mut commands: Commands,
    non_hydrated_map_elements: Query<
        (Entity, &Sort, &Handle<MapElementMeta>, &Transform),
        Without<MapElementHydrated>,
    >,
    mut ridp: ResMut<RollbackIdProvider>,
    element_assets: Res<Assets<MapElementMeta>>,
) {
    // Hydrate any newly-spawned swords
    let mut elements = non_hydrated_map_elements.iter().collect::<Vec<_>>();
    elements.sort_by_key(|x| x.1);
    for (entity, _sort, map_element_handle, transform) in elements {
        let map_element = element_assets.get(map_element_handle).unwrap();

        if let BuiltinElementKind::Sword {
            atlas_handle,
            can_rotate,
            ..
        } = &map_element.builtin
        {
            commands.entity(entity).insert(MapElementHydrated);

            commands
                .spawn()
                .insert(Rollback::new(ridp.next_id()))
                .insert(Item {
                    script: "core:sword".into(),
                })
                .insert(Name::new("Item: Sword"))
                .insert(Sword::default())
                .insert(AnimatedSprite {
                    start: 0,
                    end: 0,
                    atlas: atlas_handle.inner.clone(),
                    repeat: false,
                    fps: ATTACK_FPS,
                    ..default()
                })
                .insert_bundle(VisibilityBundle::default())
                .insert(MapRespawnPoint(transform.translation))
                .insert_bundle(TransformBundle {
                    local: *transform,
                    ..default()
                })
                .insert(map_element_handle.clone())
                .insert(KinematicBody {
                    size: Vec2::new(64.0, 16.0),
                    gravity: 1.0,
                    has_mass: true,
                    has_friction: true,
                    can_rotate: *can_rotate,
                    ..default()
                });
        }
    }
}

fn update_in_game(
    mut commands: Commands,
    players: Query<(&AnimatedSprite, &Transform, &KinematicBody), With<PlayerIdx>>,
    mut swords: Query<
        (
            Entity,
            &mut Transform,
            &mut Sword,
            &mut AnimatedSprite,
            &mut KinematicBody,
            &Handle<MapElementMeta>,
            Option<&Parent>,
            Option<&ItemUsed>,
            Option<&ItemDropped>,
        ),
        Without<PlayerIdx>,
    >,
    mut ridp: ResMut<RollbackIdProvider>,
    player_inputs: Res<PlayerInputs>,
    effects: Res<AudioChannel<EffectsChannel>>,
    element_assets: Res<Assets<MapElementMeta>>,
    collision_world: CollisionWorld,
) {
    // Helper to spawn damage regions
    let mut spawn_damage_region =
        |commands: &mut Commands, pos: Vec2, size: Vec2, owner: Entity| {
            commands
                .spawn()
                .insert(Rollback::new(ridp.next_id()))
                .insert(Transform::from_translation(pos.extend(0.0)))
                .insert(GlobalTransform::default())
                .insert(DamageRegion { size })
                .insert(DamageRegionOwner(owner))
                .insert(Lifetime::new(2.0 / 60.0));
        };

    for (
        item_ent,
        mut transform,
        mut sword,
        mut sprite,
        mut body,
        meta_handle,
        parent,
        item_used,
        item_dropped,
    ) in &mut swords
    {
        let meta = element_assets.get(meta_handle).unwrap();
        let BuiltinElementKind::Sword { sound_handle, angular_velocity, throw_velocity, arm_delay, .. } = &meta.builtin else {
            unreachable!();
        };

        // For all tiems that are being held
        if let Some(parent) = parent {
            let (player_sprite, player_transform, ..) =
                players.get(parent.get()).expect("Parent is not a player");

            // Deactivate collisions while being held
            body.is_deactivated = true;

            // Flip the sprite to match the player orientation
            let flip = player_sprite.flip_x;
            sprite.flip_x = flip;
            let flip_factor = if flip { -1.0 } else { 1.0 };
            transform.translation.x = 13.0 * flip_factor;
            transform.translation.y = 21.0;
            transform.translation.z = 0.0;
            transform.rotation = Quat::IDENTITY;

            // Reset the sword animation if we're not swinging it
            if !matches!(sword.state, SwordState::Swinging { .. }) {
                sprite.start = 4;
                sprite.end = 4;
                sprite.index = 0;
                sprite.repeat = false;
            }

            let mut next_state = None;
            match &mut sword.state {
                SwordState::Idle => (),
                SwordState::Swinging { frame } => {
                    // If we're at the end of the swinging animation
                    if sprite.index >= sprite.end.saturating_sub(sprite.start).saturating_sub(1) {
                        // Go to cooldown frames
                        next_state = Some(SwordState::Cooldown { frame: 0 });

                    // If we're still swinging
                    } else {
                        // Set the current attack frame to the animation index
                        *frame = sprite.index;
                    }

                    // TODO: Move all these constants to the builtin item config
                    match frame {
                        0 => spawn_damage_region(
                            &mut commands,
                            Vec2::new(
                                player_transform.translation.x + 20.0 * flip_factor,
                                player_transform.translation.y + 20.0,
                            ),
                            Vec2::new(30.0, 70.0),
                            parent.get(),
                        ),
                        1 => spawn_damage_region(
                            &mut commands,
                            Vec2::new(
                                player_transform.translation.x + 25.0 * flip_factor,
                                player_transform.translation.y + 20.0,
                            ),
                            Vec2::new(40.0, 50.0),
                            parent.get(),
                        ),
                        2 => spawn_damage_region(
                            &mut commands,
                            Vec2::new(
                                player_transform.translation.x + 20.0 * flip_factor,
                                player_transform.translation.y,
                            ),
                            Vec2::new(40.0, 50.0),
                            parent.get(),
                        ),
                        _ => (),
                    }

                    *frame += 1;
                }
                SwordState::Cooldown { frame } => {
                    if *frame >= COOLDOWN_FRAMES {
                        next_state = Some(SwordState::Idle);
                    } else {
                        *frame += 1;
                    }
                }
            }

            if let Some(next) = next_state {
                sword.state = next;
            }
            if item_used.is_some() {
                commands.entity(item_ent).remove::<ItemUsed>();
            }

            // If the item is being used
            if item_used.is_some() && matches!(sword.state, SwordState::Idle) {
                sprite.index = 0;
                sprite.start = 8;
                sprite.end = 12;
                sword.state = SwordState::Swinging { frame: 0 };
                if player_inputs.is_confirmed {
                    effects.play(sound_handle.clone_weak());
                }
            }
        } else {
            sword.dropped_time += 1.0 / crate::FPS as f32;
        }

        // If the item was dropped
        if let Some(dropped) = item_dropped {
            sword.dropped_time = 0.0;

            commands.entity(item_ent).remove::<ItemDropped>();
            let (.., player_transform, player_body) =
                players.get(dropped.player).expect("Parent is not a player");

            // Re-activate physics
            body.is_deactivated = false;

            // Put sword in rest position
            sprite.index = 0;
            sprite.start = 0;
            sprite.end = 0;

            if player_body.velocity != Vec2::ZERO {
                let horizontal_flip_factor = if sprite.flip_x {
                    Vec2::new(-1.0, 1.0)
                } else {
                    Vec2::ONE
                };
                body.velocity = *throw_velocity * horizontal_flip_factor + player_body.velocity;
                body.angular_velocity = *angular_velocity * if sprite.flip_x { -1.0 } else { 1.0 };
            }
            body.is_spawning = true;

            transform.translation.y = player_transform.translation.y;
            transform.translation.x = player_transform.translation.x;
            transform.translation.z = player_transform.translation.z;
        }

        if parent.is_none() && body.velocity != Vec2::ZERO && sword.dropped_time >= *arm_delay {
            for player in collision_world
                .actor_collisions(item_ent)
                .into_iter()
                .filter(|&x| players.contains(x))
            {
                commands.add(PlayerKillCommand::new(player));
            }
        }
    }
}
