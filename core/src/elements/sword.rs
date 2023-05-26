use crate::{damage::DamageRegion, prelude::*};

pub fn install(session: &mut CoreSession) {
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
    mut item_throws: CompMut<ItemThrow>,
    mut item_grabs: CompMut<ItemGrab>,
    mut respawn_points: CompMut<DehydrateOutOfBounds>,
    mut spawner_manager: SpawnerManager,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    let spawner_entities = entities
        .iter_with_bitset(&not_hydrated_bitset)
        .collect::<Vec<_>>();

    for spawner_ent in spawner_entities {
        let transform = *transforms.get(spawner_ent).unwrap();
        let element_handle = element_handles.get(spawner_ent).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        if let BuiltinElementKind::Sword {
            atlas,
            fin_anim,
            grab_offset,
            body_size,
            can_rotate,
            bounciness,
            throw_velocity,
            angular_velocity,
            ..
        } = &element_meta.builtin
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            items.insert(entity, Item);
            item_throws.insert(
                entity,
                ItemThrow::strength(*throw_velocity)
                    .with_spin(*angular_velocity)
                    .with_system(sword_drop(entity)),
            );
            item_grabs.insert(
                entity,
                ItemGrab {
                    fin_anim: *fin_anim,
                    sync_animation: false,
                    grab_offset: *grab_offset,
                },
            );
            swords.insert(entity, Sword::default());
            atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));
            respawn_points.insert(entity, DehydrateOutOfBounds(spawner_ent));
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
            spawner_manager.create_spawner(spawner_ent, vec![entity])
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
    bodies: CompMut<KinematicBody>,
    mut items_used: CompMut<ItemUsed>,
    player_indexes: Comp<PlayerIdx>,
    player_inventories: PlayerInventories,
    mut commands: Commands,
    mut player_layers: CompMut<PlayerLayers>,
    transforms: CompMut<Transform>,
    invincibles: CompMut<Invincibility>,
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
            let sprite = sprites.get_mut(entity).unwrap();
            let player_translation = transforms.get(player).unwrap().translation;
            let flip = sprite.flip_x;
            let flip_factor = if flip { -1.0 } else { 1.0 };

            let player_layer = player_layers.get_mut(player).unwrap();

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
                let sword_transform = transforms.get(entity).unwrap();

                collision_world
                    .actor_collisions_filtered(entity, |e| {
                        player_indexes.contains(e)
                            && {
                                let player_body = bodies.get(e).unwrap();
                                (player_body.velocity - body.velocity).length() >= *killing_speed
                            }
                            && invincibles.get(e).is_none()
                    })
                    .into_iter()
                    .for_each(|player| {
                        commands.add(PlayerCommand::kill(
                            player,
                            Some(sword_transform.translation.xy()),
                        ))
                    });
            }
        }
    }
}

fn sword_drop(entity: Entity) -> System {
    (move |mut swords: CompMut<Sword>, mut sprites: CompMut<AtlasSprite>| {
        // Put sword in rest position
        sprites.get_mut(entity).unwrap().index = 0;
        *swords.get_mut(entity).unwrap() = default();
    })
    .system()
}
