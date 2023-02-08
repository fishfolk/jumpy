use crate::{
    physics::collisions::{Actor, Collider, TileCollisionKind},
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
pub struct Bullet {
    pub direction: f32,
}

impl Default for Bullet {
    fn default() -> Self {
        Self { direction: 1.0 }
    }
}

/// Component containing the bullet's metadata handle.
#[derive(Deref, DerefMut, TypeUlid, Clone)]
#[ulid = "01GR1WH27X84VX22G0JY9J71PC"]
pub struct BulletHandle(pub Handle<BulletMeta>);

fn hydrate(
    entities: Res<Entities>,
    mut actors: CompMut<Actor>,
    mut colliders: CompMut<Collider>,
    mut lifetimes: CompMut<Lifetime>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    bullet_assets: BevyAssets<BulletMeta>,
    bullet_handles: Comp<BulletHandle>,
) {
    // We consider all entities with bullet handles, but that don't have physics actors on them to
    // be non-hydrated.
    let mut not_hydrated_bitset = actors.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(bullet_handles.bitset());

    let bullets = entities
        .iter_with_bitset(&not_hydrated_bitset)
        .collect::<Vec<_>>();

    for entity in bullets {
        let bullet_handle = bullet_handles.get(entity).unwrap();
        let Some(bullet_meta) = bullet_assets.get(&bullet_handle.get_bevy_handle()) else {
                continue;
            };

        let BulletMeta {
            atlas,
            body_diameter,
            ..
        } = &bullet_meta;

        atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));

        // Setup custom collider
        actors.insert(entity, Actor);
        colliders.insert(
            entity,
            Collider {
                shape: ColliderShape::Circle {
                    diameter: *body_diameter,
                },
                ..default()
            },
        );

        lifetimes.insert(entity, Lifetime::new(bullet_meta.lifetime));
    }
}

fn update(
    entities: Res<Entities>,
    mut commands: Commands,
    bullet_handles: Comp<BulletHandle>,
    bullet_assets: BevyAssets<BulletMeta>,

    player_indexes: Comp<PlayerIdx>,
    collision_world: CollisionWorld,
    mut player_events: ResMut<PlayerEvents>,
    mut transforms: CompMut<Transform>,
    mut bullets: CompMut<Bullet>,
    mut audio_events: ResMut<AudioEvents>,
) {
    for (entity, (bullet, bullet_handle)) in entities.iter_with((&mut bullets, &bullet_handles)) {
        let Some(bullet_meta) = bullet_assets.get(&bullet_handle.get_bevy_handle()) else {
            continue;
        };

        let BulletMeta {
            velocity,
            body_diameter,
            explosion_fps,
            explosion_volume,
            explosion_sound,
            explosion_atlas,
            explosion_frames,
            explosion_lifetime,
            ..
        } = bullet_meta;

        // Move bullet
        let position = {
            let position = transforms.get_mut(entity).unwrap();
            position.translation += bullet.direction * velocity.extend(0.0);
            *position
        };

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
        let hit_solid = collision_world.tile_collision(
            position,
            ColliderShape::Circle {
                diameter: *body_diameter,
            },
        ) != TileCollisionKind::EMPTY;

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
                                frames: (0..explosion_frames).collect(),
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
