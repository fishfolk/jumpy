use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("flappy_jellyfish"))]
#[repr(C)]
pub struct FlappyJellyfishMeta {
    pub atlas: Handle<Atlas>,
    pub body_size: Vec2,
    pub start_frame: u32,
    pub end_frame: u32,
    pub fps: f32,
    pub spawn_offset: Vec2,
    pub explosion_atlas: Handle<Atlas>,
    pub explosion_lifetime: f32,
    pub explosion_frames: u32,
    pub explosion_fps: f32,
    pub explosion_volume: f64,
    pub explosion_sound: Handle<AudioSource>,
    pub damage_region_size: Vec2,
    pub damage_region_lifetime: f32,
}

impl FlappyJellyfishMeta {
    pub fn frames(&self) -> SVec<u32> {
        (self.start_frame..self.end_frame).collect()
    }
}

pub trait FlappyJellyfishMetaSchemaExts {
    /// Try to cast the `asset` to a `JellyfishMeta` and get the
    /// `FlappyJellyfishMeta` from it.
    fn try_get_flappy_meta(&self) -> Option<Handle<FlappyJellyfishMeta>>;
}

impl FlappyJellyfishMetaSchemaExts for SchemaBox {
    fn try_get_flappy_meta(&self) -> Option<Handle<FlappyJellyfishMeta>> {
        self.try_cast_ref::<JellyfishMeta>()
            .ok()
            .map(|m| m.flappy_meta)
    }
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PostUpdate, control_flappy_jellyfish)
        .add_system_to_stage(CoreStage::PostUpdate, explode_flappy_jellyfish);
}

#[derive(Clone, Copy, Debug, Default, HasSchema)]
pub struct FlappyJellyfish {
    pub owner: Entity,
    pub jellyfish: Entity,
}

/// A player with a jellyfish is holding shoot. Take control of the flappy if
/// one exists or spawn one.
pub fn spawn_or_take_control(
    owner: Entity,
    jellyfish_ent: Entity,
    flappy_ent: Option<Entity>,
) -> StaticSystem<(), ()> {
    (move |world: &World| {
        if let Some(flappy_ent) = flappy_ent {
            take_control(owner, flappy_ent).run(world, ());
        } else {
            {
                let mut jellyfishes = world.components.get::<Jellyfish>().borrow_mut();
                let Some(jellyfish) = jellyfishes.get_mut(jellyfish_ent) else {
                    return;
                };
                if jellyfish.ammo == 0 {
                    return;
                }
                jellyfish.ammo -= 1;
            }
            spawn(owner, jellyfish_ent).run(world, ());
        }
    })
    .system()
}

/// Take control of the flappy associated with a jellyfish for a player.
fn take_control(owner: Entity, flappy_ent: Entity) -> StaticSystem<(), ()> {
    (move |mut player_driving: CompMut<PlayerDrivingJellyfish>| {
        player_driving.insert(owner, PlayerDrivingJellyfish { flappy: flappy_ent });
    })
    .system()
}

/// Spawn a flappy jellyfish.
fn spawn(owner: Entity, jellyfish_ent: Entity) -> StaticSystem<(), ()> {
    (move |mut entities: ResMut<Entities>,
           element_handles: Comp<ElementHandle>,
           mut player_driving: CompMut<PlayerDrivingJellyfish>,
           mut jellyfishes: CompMut<Jellyfish>,
           mut flappy_jellyfishes: CompMut<FlappyJellyfish>,
           mut bodies: CompMut<KinematicBody>,
           assets: Res<AssetServer>,
           mut atlas_sprites: CompMut<AtlasSprite>,
           mut animated_sprites: CompMut<AnimatedSprite>,
           mut transforms: CompMut<Transform>,
           mut camera_subjects: CompMut<CameraSubject>| {
        let Some(flappy_meta) = element_handles
            .get(jellyfish_ent)
            .map(|element_h| assets.get(element_h.0))
            .map(|element_meta| assets.get(element_meta.data))
            .as_deref()
            .and_then(SchemaBox::try_get_flappy_meta)
            .map(|flappy_h| assets.get(flappy_h))
        else {
            return;
        };

        let flappy_ent = entities.create();

        player_driving.insert(owner, PlayerDrivingJellyfish { flappy: flappy_ent });
        if let Some(jellyfish) = jellyfishes.get_mut(jellyfish_ent) {
            jellyfish.flappy = Some(flappy_ent);
        }

        flappy_jellyfishes.insert(
            flappy_ent,
            FlappyJellyfish {
                owner,
                jellyfish: jellyfish_ent,
            },
        );
        bodies.insert(
            flappy_ent,
            KinematicBody {
                shape: ColliderShape::Rectangle {
                    size: flappy_meta.body_size,
                },
                has_mass: true,
                gravity: GRAVITY,
                fall_through: true,
                is_controlled: true,
                ..default()
            },
        );
        atlas_sprites.insert(flappy_ent, AtlasSprite::new(flappy_meta.atlas));
        animated_sprites.insert(
            flappy_ent,
            AnimatedSprite {
                frames: flappy_meta.frames(),
                fps: flappy_meta.fps,
                repeat: true,
                ..default()
            },
        );
        let mut transf = *transforms.get(owner).unwrap();
        transf.translation += flappy_meta.spawn_offset.extend(0.0);
        transforms.insert(flappy_ent, transf);
        camera_subjects.insert(flappy_ent, default());
    })
    .system()
}

const SPEED_X: f32 = 324.0;
const SPEED_JUMP: f32 = 3.5;
const GRAVITY: f32 = 0.1;
const MIN_SPEED: Vec2 = vec2(-SPEED_X, -4.0);
const MAX_SPEED: Vec2 = vec2(SPEED_X, 4.0);

fn control_flappy_jellyfish(
    time: Res<Time>,
    entities: Res<Entities>,
    player_driving: Comp<PlayerDrivingJellyfish>,
    player_indexes: Comp<PlayerIdx>,
    player_inputs: Res<MatchInputs>,
    mut explode_flappies: CompMut<ExplodeFlappyJellyfish>,
    mut bodies: CompMut<KinematicBody>,
    flappy_jellyfishes: Comp<FlappyJellyfish>,
) {
    let t = time.delta_seconds();

    for (_player_ent, (driving, player_idx)) in
        entities.iter_with((&player_driving, &player_indexes))
    {
        let owner_control = player_inputs.players[player_idx.0 as usize].control;

        if owner_control.grab_just_pressed {
            explode_flappies.insert(driving.flappy, ExplodeFlappyJellyfish);
            continue;
        }

        let Some(body) = bodies.get_mut(driving.flappy) else {
            continue;
        };

        if owner_control.left == owner_control.right {
            body.velocity.x = 0.0;
        } else if owner_control.left > f32::EPSILON {
            body.velocity.x = -owner_control.left * SPEED_X * t;
        } else if owner_control.right > f32::EPSILON {
            body.velocity.x = owner_control.right * SPEED_X * t;
        }

        if owner_control.jump_just_pressed {
            body.velocity.y += SPEED_JUMP;
        }
    }

    for (_, (_, body)) in entities.iter_with((&flappy_jellyfishes, &mut bodies)) {
        body.velocity = body.velocity.clamp(MIN_SPEED, MAX_SPEED);
    }
}

/// A marker component for flappy jellyfish to indicate that it should explode.
#[derive(Clone, Copy, Debug, Default, HasSchema)]
pub struct ExplodeFlappyJellyfish;

/// Explode flappy jellyfish that either have the `KillFlappyJellyfish` marker
/// or that have a dead owner.
fn explode_flappy_jellyfish(
    mut entities: ResMut<Entities>,
    explode_flappies: Comp<ExplodeFlappyJellyfish>,
    flappy_jellyfishes: Comp<FlappyJellyfish>,
    mut jellyfishes: CompMut<Jellyfish>,
    player_indexes: Comp<PlayerIdx>,
    invincibles: Comp<Invincibility>,
    bodies: Comp<KinematicBody>,
    map: Res<LoadedMap>,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,
    mut transforms: CompMut<Transform>,
    mut audio_center: ResMut<AudioCenter>,
    mut trauma_events: ResMut<CameraTraumaEvents>,
    mut sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut damage_regions: CompMut<DamageRegion>,
    mut lifetimes: CompMut<Lifetime>,
    mut dehydrate_jellyfish: CompMut<DehydrateJellyfish>,
) {
    // Collect the hitboxes of all players
    let mut player_hitboxes = SmallVec::<[Rect; MAX_PLAYERS]>::with_capacity(MAX_PLAYERS);
    player_hitboxes.extend(
        entities
            .iter_with((&player_indexes, &transforms, &bodies))
            .filter(|(player_ent, _)| !invincibles.contains(*player_ent))
            .map(|(_, (_, transform, body))| body.bounding_box(*transform)),
    );

    let mut explode_flappy_entities =
        SmallVec::<[Entity; 8]>::with_capacity(flappy_jellyfishes.bitset().bit_count());

    for (flappy_ent, (_flappy_jellyfish, transform, body)) in
        entities.iter_with((&flappy_jellyfishes, &transforms, &bodies))
    {
        // If flappy has the explode marker
        if explode_flappies.contains(flappy_ent) {
            explode_flappy_entities.push(flappy_ent);
            continue;
        }
        // If the flappy collides with any player
        let flappy_hitbox = body.bounding_box(*transform);
        if player_hitboxes.iter().any(|b| b.overlaps(&flappy_hitbox)) {
            explode_flappy_entities.push(flappy_ent);
            continue;
        }
        // If the flappy is out of bounds
        if map.is_out_of_bounds(&transform.translation) {
            explode_flappy_entities.push(flappy_ent);
            continue;
        }
    }

    for flappy_ent in explode_flappy_entities {
        let Some(flappy) = flappy_jellyfishes.get(flappy_ent) else {
            continue;
        };
        let Some(jellyfish) = jellyfishes.get_mut(flappy.jellyfish) else {
            continue;
        };
        jellyfish.flappy.take();

        /*
         * Get explosion data
         */

        let Some(flappy_meta) = element_handles
            .get(flappy.jellyfish)
            .map(|element_h| assets.get(element_h.0))
            .map(|element_meta| assets.get(element_meta.data))
            .as_deref()
            .and_then(SchemaBox::try_get_flappy_meta)
            .map(|flappy_h| assets.get(flappy_h))
        else {
            return;
        };

        let Some(mut explosion_transform) = transforms.get(flappy_ent).copied() else {
            return;
        };
        explosion_transform.translation.z = -10.0;

        /*
         * Setup the explosion
         */

        entities.kill(flappy_ent);

        audio_center.play_sound(flappy_meta.explosion_sound, flappy_meta.explosion_volume);

        trauma_events.send(5.0);

        // Explosion animation entity
        {
            let explosion_ent = entities.create();
            transforms.insert(explosion_ent, explosion_transform);
            sprites.insert(
                explosion_ent,
                AtlasSprite {
                    atlas: flappy_meta.explosion_atlas,
                    ..default()
                },
            );
            animated_sprites.insert(
                explosion_ent,
                AnimatedSprite {
                    frames: (0..flappy_meta.explosion_frames).collect(),
                    fps: flappy_meta.explosion_fps,
                    repeat: false,
                    ..default()
                },
            );
            lifetimes.insert(explosion_ent, Lifetime::new(flappy_meta.explosion_lifetime));
        }

        // Damage region entity
        {
            let damage_ent = entities.create();
            transforms.insert(damage_ent, explosion_transform);
            damage_regions.insert(
                damage_ent,
                DamageRegion {
                    size: flappy_meta.damage_region_size,
                },
            );
            lifetimes.insert(
                damage_ent,
                Lifetime::new(flappy_meta.damage_region_lifetime),
            );
        }

        /*
         * Despawn the jellyfish if out of ammo
         */

        if jellyfish.ammo == 0 {
            dehydrate_jellyfish.insert(
                flappy.jellyfish,
                DehydrateJellyfish {
                    owner: flappy.owner,
                },
            );
        }
    }
}
