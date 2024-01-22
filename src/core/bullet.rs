//! Generic bullet implementation.
//!
//! These components are used for things like the musket and sniper rifle bullets.

use crate::prelude::*;

pub fn game_plugin(game: &mut Game) {
    BulletMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

/// Install this module.
pub fn session_plugin(session: &mut Session) {
    Bullet::register_schema();
    BulletHandle::register_schema();

    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update);
}

/// Bullet component.
#[derive(Clone, Debug, HasSchema, Default, Copy)]
#[repr(C)]
pub struct Bullet {
    /// The direction that the bullet is moving.
    pub direction: Vec2,
    /// The player entity that shot the bullet.
    pub owner: Entity,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[type_data(metadata_asset("bullet"))]
#[repr(C)]
pub struct BulletMeta {
    pub speed: f32,
    pub body_diameter: f32,
    pub atlas: Handle<Atlas>,

    pub lifetime: f32,
    pub explosion_fps: f32,
    pub explosion_volume: f64,
    pub explosion_lifetime: f32,
    pub explosion_frames: u32,
    pub explosion_atlas: Handle<Atlas>,
    pub explosion_sound: Handle<AudioSource>,
}

/// Component containing the bullet's metadata handle.
#[derive(Deref, DerefMut, HasSchema, Default, Clone)]
#[repr(C)]
pub struct BulletHandle(pub Handle<BulletMeta>);

/// Hydrate bullets.
fn hydrate(
    entities: Res<Entities>,
    mut actors: CompMut<Actor>,
    mut colliders: CompMut<Collider>,
    mut lifetimes: CompMut<Lifetime>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    bullet_handles: Comp<BulletHandle>,
    assets: Res<AssetServer>,
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
        let bullet_meta = assets.get(bullet_handle.0);

        let BulletMeta {
            atlas,
            body_diameter,
            ..
        } = &*bullet_meta;

        atlas_sprites.insert(entity, AtlasSprite::new(*atlas));

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

/// Update bullets.
fn update(
    entities: Res<Entities>,
    mut commands: Commands,
    bullet_handles: Comp<BulletHandle>,
    player_indexes: Comp<PlayerIdx>,
    collision_world: CollisionWorld,
    mut transforms: CompMut<Transform>,
    mut bullets: CompMut<Bullet>,
    mut audio_center: ResMut<AudioCenter>,
    invincibles: CompMut<Invincibility>,
    mut emote_regions: CompMut<EmoteRegion>,
    asset_server: Res<AssetServer>,
) {
    for (entity, (bullet, bullet_handle)) in entities.iter_with((&mut bullets, &bullet_handles)) {
        let bullet_meta = asset_server.get(bullet_handle.0);

        let BulletMeta {
            speed,
            body_diameter,
            explosion_fps,
            explosion_volume,
            explosion_sound,
            explosion_atlas,
            explosion_frames,
            explosion_lifetime,
            ..
        } = &*bullet_meta;

        // Move bullet
        let position = {
            let position = transforms.get_mut(entity).unwrap();
            position.translation += (bullet.direction * *speed).extend(0.0);

            let emote_size = Vec2::new(*body_diameter * 6.0, *body_diameter * 3.5);
            emote_regions.insert(entity, EmoteRegion::basic(Emote::Alarm, emote_size, true));

            *position
        };

        // Check actor collisions
        let mut hit_player = false;
        collision_world
            .actor_collisions_filtered(entity, |e| {
                player_indexes.contains(e) && invincibles.get(e).is_none()
            })
            .into_iter()
            .filter(|player| *player != bullet.owner)
            .for_each(|player| {
                hit_player = true;
                commands.add(PlayerCommand::kill(player, Some(position.translation.xy())));
            });

        // check solid tile collisions
        let hit_solid = collision_world.tile_collision(
            position,
            ColliderShape::Circle {
                diameter: *body_diameter,
            },
        ) == TileCollisionKind::Solid;

        // Bullet hit something
        if hit_player || hit_solid {
            audio_center.play_sound(*explosion_sound, *explosion_volume);

            let mut explosion_transform = *transforms.get(entity).unwrap();
            explosion_transform.translation.z += 1.0;

            let explosion_fps = *explosion_fps;
            let explosion_frames = *explosion_frames;
            let explosion_lifetime = *explosion_lifetime;
            let explosion_atlas = *explosion_atlas;

            commands.add(
                move |mut entities: ResMutInit<Entities>,
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
                                atlas: explosion_atlas,
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
