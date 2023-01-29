use crate::{
    physics::collisions::{Actor, Collider, TileCollision},
    prelude::*,
};

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update);
}

#[derive(Clone, Debug, TypeUlid, Copy)]
#[ulid = "01GQX3KM2A4WPV2NKJNG85TJ3P"]
pub struct MusketBullet {
    pub direction: f32,
}

impl Default for MusketBullet {
    fn default() -> Self {
        Self { direction: 1.0 }
    }
}

fn hydrate(
    mut items: CompMut<Item>,
    mut actors: CompMut<Actor>,
    mut entities: ResMut<Entities>,
    mut colliders: CompMut<Collider>,
    mut lifetimes: CompMut<Lifetime>,
    mut bullets: CompMut<MusketBullet>,
    mut transforms: CompMut<Transform>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    element_assets: BevyAssets<ElementMeta>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    let spawners = entities
        .iter_with_bitset(&not_hydrated_bitset)
        .collect::<Vec<_>>();

    for spawner_ent in spawners {
        let element_handle = element_handles.get(spawner_ent).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
                continue;
            };

        if let BuiltinElementKind::MusketBullet {
            atlas,
            body_size,
            body_offset,
            ..
        } = &element_meta.builtin
        {
            let transform = *transforms.get(spawner_ent).unwrap();
            let bullet = if let Some(bullet) = bullets.get(spawner_ent) {
                *bullet
            } else {
                MusketBullet::default()
            };

            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            items.insert(entity, Item);
            bullets.insert(entity, bullet);

            transforms.insert(entity, transform);
            hydrated.insert(entity, MapElementHydrated);
            element_handles.insert(entity, element_handle.clone());
            atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));

            // Setup custom collider
            actors.insert(entity, Actor);
            colliders.insert(
                entity,
                Collider {
                    width: body_size.x,
                    height: body_size.y,
                    pos: transform.translation.truncate() + *body_offset,
                    ..default()
                },
            );

            lifetimes.insert(entity, Lifetime::new(1.0));
        }
    }
}

fn update(
    entities: Res<Entities>,
    mut commands: Commands,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,

    player_indexes: Comp<PlayerIdx>,
    mut collision_world: CollisionWorld,
    mut player_events: ResMut<PlayerEvents>,

    mut bullets: CompMut<MusketBullet>,
    mut transforms: CompMut<Transform>,
    mut audio_events: ResMut<AudioEvents>,
) {
    for (entity, (bullet, element_handle)) in entities.iter_with((&mut bullets, &element_handles)) {
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        let BuiltinElementKind::MusketBullet {
            velocity,
            body_size,
            explosion_fps,
            explosion_volume,
            explosion_sound,
            explosion_atlas,
            explosion_frames,
            explosion_lifetime,
            ..
        } = &element_meta.builtin else {
            unreachable!();
        };

        // Move bullet
        let position = transforms.get_mut(entity).unwrap();
        position.translation += bullet.direction * velocity.extend(0.0);
        collision_world.set_actor_position(entity, position.translation.truncate());

        // Check actor collisions
        let mut hit_player = false;
        collision_world
            .actor_collisions(entity)
            .into_iter()
            .filter(|&x| player_indexes.contains(x))
            .for_each(|player| {
                hit_player = true;
                player_events.kill(player);
            });

        // check solid tile collisions
        let hit_solid = collision_world.collide_solids(
            position.translation.truncate(),
            body_size.x,
            body_size.y,
        ) != TileCollision::EMPTY;

        // Bullet hit something
        if hit_player || hit_solid {
            audio_events.play(explosion_sound.clone(), *explosion_volume);

            let mut explosion_transform = *transforms.get(entity).unwrap();
            explosion_transform.translation.z += 1.0;

            let explosion_fps = *explosion_fps;
            let explosion_frames = *explosion_frames;
            let explosion_lifetime = *explosion_lifetime;
            let explosion_atlas = explosion_atlas.clone();

            commands.add(
                move |mut entities: ResMut<Entities>,
                      mut transforms: CompMut<Transform>,
                      mut lifetimes: CompMut<Lifetime>,
                      mut sprites: CompMut<AtlasSprite>,
                      mut animated_sprites: CompMut<AnimatedSprite>| {
                    // Despawn the bullet
                    entities.kill(entity);

                    // spawn bullet explosion animation
                    {
                        let ent = entities.create();
                        transforms.insert(ent, explosion_transform);
                        sprites.insert(
                            ent,
                            AtlasSprite {
                                atlas: explosion_atlas.clone(),
                                ..default()
                            },
                        );

                        animated_sprites.insert(
                            ent,
                            AnimatedSprite {
                                start: 0,
                                end: explosion_frames,
                                fps: explosion_fps,
                                repeat: false,
                                ..default()
                            },
                        );
                        lifetimes.insert(ent, Lifetime::new(explosion_lifetime));
                    }
                },
            );
        }
    }
}
