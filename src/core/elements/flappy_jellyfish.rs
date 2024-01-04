use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("flappy_jellyfish"))]
#[repr(C)]
pub struct FlappyJellyfishMeta {
    pub atlas: Handle<Atlas>,
    pub start_frame: u32,
    pub end_frame: u32,
    pub fps: f32,
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

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PostUpdate, move_flappy_jellyfish);
}

#[derive(Clone, Copy, Debug, HasSchema, Default)]
pub struct FlappyJellyfish {
    pub owner: Entity,
}

pub fn spawn(owner: Entity, jellyfish_ent: Entity) -> StaticSystem<(), ()> {
    (move |mut entities: ResMut<Entities>,
           element_handles: Comp<ElementHandle>,
           mut driving_jellyfishes: CompMut<DrivingJellyfish>,
           mut flappy_jellyfishes: CompMut<FlappyJellyfish>,
           mut fall_velocities: CompMut<FallVelocity>,
           assets: Res<AssetServer>,
           mut atlas_sprites: CompMut<AtlasSprite>,
           mut animated_sprites: CompMut<AnimatedSprite>,
           mut transforms: CompMut<Transform>| {
        let Some(flappy_meta) = element_handles
            .get(jellyfish_ent)
            .map(|element_h| assets.get(element_h.0))
            .map(|element_meta| assets.get(element_meta.data))
            .and_then(JellyfishMeta::get_flappy_meta_from_asset)
            .map(|flappy_h| assets.get(flappy_h))
        else {
            return;
        };

        let flappy_ent = entities.create();
        driving_jellyfishes.insert(
            jellyfish_ent,
            DrivingJellyfish {
                owner,
                flappy: flappy_ent,
            },
        );
        flappy_jellyfishes.insert(flappy_ent, FlappyJellyfish { owner });
        fall_velocities.insert(flappy_ent, FallVelocity::default());
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
        transf.translation.y += 75.0;
        transforms.insert(flappy_ent, transf);
        debug!("FLAPPY JELLYFISH | spawned");
    })
    .system()
}

pub fn kill(jellyfish: Entity, flappy: Entity) -> StaticSystem<(), ()> {
    (move |mut entities: ResMut<Entities>,
           element_handles: Comp<ElementHandle>,
           assets: Res<AssetServer>,
           mut transforms: CompMut<Transform>,
           mut audio_events: ResMut<AudioEvents>,
           mut trauma_events: ResMut<CameraTraumaEvents>,
           mut sprites: CompMut<AtlasSprite>,
           mut animated_sprites: CompMut<AnimatedSprite>,
           mut damage_regions: CompMut<DamageRegion>,
           mut lifetimes: CompMut<Lifetime>| {
        let Some(flappy_meta) = element_handles
            .get(jellyfish)
            .map(|element_h| assets.get(element_h.0))
            .map(|element_meta| assets.get(element_meta.data))
            .and_then(JellyfishMeta::get_flappy_meta_from_asset)
            .map(|flappy_h| assets.get(flappy_h))
        else {
            return;
        };

        let Some(mut explosion_transform) = transforms.get(flappy).copied() else {
            return;
        };
        explosion_transform.translation.z = -10.0;

        entities.kill(flappy);
        debug!("FLAPPY JELLYFISH | despawned");

        audio_events.play(flappy_meta.explosion_sound, flappy_meta.explosion_volume);

        trauma_events.send(5.0);

        // Explosion
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

        // Damage region
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
    })
    .system()
}

#[derive(Clone, Copy, Default, Deref, DerefMut, HasSchema)]
pub struct FallVelocity(f32);

const SPEED_X: f32 = 200.0;
const SPEED_JUMP: f32 = 500.0;
const GRAVITY: f32 = -700.0;
const MAX_SPEED_Y_ABS: f32 = 300.0;

pub fn move_flappy_jellyfish(
    entities: Res<Entities>,
    flappy_jellyfishes: Comp<FlappyJellyfish>,
    player_indexes: Comp<PlayerIdx>,
    player_inputs: Res<MatchInputs>,
    time: Res<Time>,
    mut fall_velocities: CompMut<FallVelocity>,
    mut transforms: CompMut<Transform>,
) {
    let t = time.delta_seconds();

    for (_e, (&FlappyJellyfish { owner }, fall_velocity, transform)) in
        entities.iter_with((&flappy_jellyfishes, &mut fall_velocities, &mut transforms))
    {
        let Some(owner_idx) = player_indexes.get(owner).cloned() else {
            continue;
        };
        let owner_control = player_inputs.players[owner_idx.0 as usize].control;

        let mut delta_pos = Vec2::ZERO;

        if owner_control.left != owner_control.right {
            delta_pos.x -= owner_control.left * SPEED_X * t;
            delta_pos.x += owner_control.right * SPEED_X * t;
        }

        if owner_control.jump_just_pressed {
            **fall_velocity += SPEED_JUMP;
        }

        // Velocity formula: `vₜ = vᵢ + tg`
        **fall_velocity = (**fall_velocity + t * GRAVITY).clamp(-MAX_SPEED_Y_ABS, MAX_SPEED_Y_ABS);

        // Displacement formula: `y = gt²/2 + tvₜ`
        delta_pos.y += GRAVITY * t.powi(2) / 2.0 + **fall_velocity * t;

        transform.translation += delta_pos.extend(0.0);
    }
}
