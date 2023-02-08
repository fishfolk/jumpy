use crate::{damage::DamageRegion, prelude::*};

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update);
}

#[derive(Copy, Clone, Debug, TypeUlid, Default)]
#[ulid = "01GP9NA6ZQGMC0YY9K2XFJH9KA"]
pub struct Sword {
    pub state: SwordState,
    pub dropped_time: f32,
}

#[derive(Default, Clone, Copy, Debug)]
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

fn hydrate(
    game_meta: Res<CoreMetaArc>,
    mut entities: ResMut<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut swords: CompMut<Sword>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    mut items: CompMut<Item>,
    mut respawn_points: CompMut<MapRespawnPoint>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    let spawners = entities
        .iter_with_bitset(&not_hydrated_bitset)
        .collect::<Vec<_>>();

    for spawner_ent in spawners {
        let transform = *transforms.get(spawner_ent).unwrap();
        let element_handle = element_handles.get(spawner_ent).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        if let BuiltinElementKind::Sword {
            atlas,
            body_size,
            can_rotate,
            bounciness,
            ..
        } = &element_meta.builtin
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            items.insert(entity, Item);
            swords.insert(entity, Sword::default());
            atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));
            respawn_points.insert(entity, MapRespawnPoint(transform.translation));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle.clone());
            hydrated.insert(entity, MapElementHydrated);
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Rectangle { size: *body_size },
                    has_mass: true,
                    has_friction: true,
                    can_rotate: *can_rotate,
                    bounciness: *bounciness,
                    gravity: game_meta.physics.gravity,
                    ..default()
                },
            );
        }
    }
}

#[derive(Clone, TypeUlid, Default, Deref, DerefMut)]
#[ulid = "01GP9W74TET7QMND26S7G7NCA9"]
struct PendingDamageRegions(Vec<(Vec2, Vec2, Entity)>);

fn update(
    entities: Res<Entities>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    collision_world: CollisionWorld,
    mut audio_events: ResMut<AudioEvents>,
    mut swords: CompMut<Sword>,
    mut sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut items_used: CompMut<ItemUsed>,
    mut items_dropped: CompMut<ItemDropped>,
    mut attachments: CompMut<PlayerBodyAttachment>,
    player_indexes: Comp<PlayerIdx>,
    player_inventories: PlayerInventories,
    mut player_events: ResMut<PlayerEvents>,
    mut commands: Commands,
    mut player_layers: CompMut<PlayerLayers>,
    mut transforms: CompMut<Transform>,
) {
    for (entity, (sword, element_handle)) in entities.iter_with((&mut swords, &element_handles)) {
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        // Helper to spawn a damage region for the sword attack
        let mut spawn_damage_region = |pos: Vec3, size: Vec2, owner: Entity| {
            commands.add(
                move |mut entities: ResMut<Entities>,
                      mut transforms: CompMut<Transform>,
                      mut damage_regions: CompMut<DamageRegion>,
                      mut damage_region_owners: CompMut<DamageRegionOwner>,
                      mut lifetimes: CompMut<Lifetime>| {
                    let entity = entities.create();

                    transforms.insert(entity, Transform::from_translation(pos));
                    damage_regions.insert(entity, DamageRegion { size });
                    damage_region_owners.insert(entity, DamageRegionOwner(owner));
                    lifetimes.insert(entity, Lifetime::new(2.0 / 60.0));
                },
            );
        };

        let BuiltinElementKind::Sword {
            cooldown_frames,
            sound,
            sound_volume,
            grab_offset,
            throw_velocity,
            angular_velocity,
            fin_anim,
            killing_speed,
            ..
        } = &element_meta.builtin else {
            unreachable!();
        };

        // If the item is being held
        if let Some(inventory) = player_inventories
            .iter()
            .find_map(|x| x.filter(|x| x.inventory == entity))
        {
            let player = inventory.player;
            let body = bodies.get_mut(entity).unwrap();
            let sprite = sprites.get_mut(entity).unwrap();
            let player_translation = transforms.get(player).unwrap().translation;
            let flip = sprite.flip_x;
            let flip_factor = if flip { -1.0 } else { 1.0 };

            let player_layer = player_layers.get_mut(player).unwrap();
            player_layer.fin_anim = *fin_anim;

            // Deactivate collisions while being held
            body.is_deactivated = true;

            attachments.insert(
                entity,
                PlayerBodyAttachment {
                    player,
                    offset: grab_offset.extend(1.0),
                    sync_animation: false,
                },
            );

            // Reset the sword animation if we're not swinging it
            if !matches!(sword.state, SwordState::Swinging { .. }) {
                sprite.index = 4;
            }

            let mut next_state = None;
            match &mut sword.state {
                SwordState::Idle => (),
                SwordState::Swinging { frame } => {
                    // If we're at the end of the swinging animation
                    if sprite.index >= 11 {
                        player_layer.fin_offset = Vec2::ZERO;
                        // Go to cooldown frames
                        next_state = Some(SwordState::Cooldown { frame: 0 });

                    // If we're still swinging
                    } else {
                        // Set the current attack frame to the animation index
                        sprite.index = 8 + *frame / 3;
                    }

                    // TODO: Move all these constants to the builtin item config
                    match *frame / 3 {
                        0 => {
                            spawn_damage_region(
                                Vec3::new(
                                    player_translation.x + 20.0 * flip_factor,
                                    player_translation.y + 20.0,
                                    player_translation.z,
                                ),
                                Vec2::new(30.0, 70.0),
                                player,
                            );

                            player_layer.fin_offset = vec2(-1.0, 2.0);
                        }
                        1 => {
                            spawn_damage_region(
                                Vec3::new(
                                    player_translation.x + 25.0 * flip_factor,
                                    player_translation.y + 20.0,
                                    player_translation.z,
                                ),
                                Vec2::new(40.0, 50.0),
                                player,
                            );
                            player_layer.fin_offset = vec2(0.0, -1.0);
                        }
                        2 => {
                            spawn_damage_region(
                                Vec3::new(
                                    player_translation.x + 20.0 * flip_factor,
                                    player_translation.y,
                                    player_translation.z,
                                ),
                                Vec2::new(40.0, 40.0),
                                player,
                            );
                            player_layer.fin_offset = vec2(0.0, -2.0);
                        }
                        _ => (),
                    }

                    *frame += 1;
                }
                SwordState::Cooldown { frame } => {
                    if *frame >= *cooldown_frames {
                        next_state = Some(SwordState::Idle);
                    } else {
                        *frame += 1;
                    }
                }
            }

            if let Some(next) = next_state {
                sword.state = next;
            }

            // If the item is being used
            let item_used = items_used.get(entity).is_some();
            if item_used {
                items_used.remove(entity);
                if matches!(sword.state, SwordState::Idle) {
                    sprite.index = 8;
                    sword.state = SwordState::Swinging { frame: 0 };
                    audio_events.play(sound.clone(), *sound_volume);
                }
            }
        } else {
            let body = bodies.get(entity).unwrap();
            sword.dropped_time += 1.0 / crate::FPS;

            if body.velocity.length() >= *killing_speed {
                collision_world
                    .actor_collisions(entity)
                    .into_iter()
                    .filter(|&x| {
                        player_indexes.contains(x) && {
                            let player_body = bodies.get(x).unwrap();
                            (player_body.velocity - body.velocity).length() >= *killing_speed
                        }
                    })
                    .for_each(|player| player_events.kill(player));
            }
        }

        // If the item was dropped
        if let Some(dropped) = items_dropped.get(entity).copied() {
            attachments.remove(entity);
            let player = dropped.player;
            sword.dropped_time = 0.0;

            items_dropped.remove(entity);
            let player_translation = transforms.get(dropped.player).unwrap().translation;
            let player_velocity = bodies.get(player).unwrap().velocity;

            let body = bodies.get_mut(entity).unwrap();
            let sprite = sprites.get_mut(entity).unwrap();

            // Re-activate physics
            body.is_deactivated = false;

            // Put sword in rest position
            sprite.index = 0;

            let horizontal_flip_factor = if sprite.flip_x {
                Vec2::new(-1.0, 1.0)
            } else {
                Vec2::ONE
            };

            if player_velocity != Vec2::ZERO {
                body.velocity = *throw_velocity * horizontal_flip_factor + player_velocity;
                body.angular_velocity = *angular_velocity * if sprite.flip_x { -1.0 } else { 1.0 };
            }
            body.is_spawning = true;

            let transform = transforms.get_mut(entity).unwrap();
            transform.translation =
                player_translation + (*grab_offset * horizontal_flip_factor).extend(0.0);
        }
    }
}
