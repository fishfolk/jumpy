use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("flappy_jellyfish"))]
#[repr(C)]
pub struct FlappyJellyfishMeta {
    pub atlas: Handle<Atlas>,
    pub start_frame: u32,
    pub end_frame: u32,
    pub fps: f32,
}

impl FlappyJellyfishMeta {
    pub fn frames(&self) -> SVec<u32> {
        (self.start_frame..self.end_frame).collect()
    }
}

#[derive(Clone, Copy, Debug, HasSchema, Default)]
pub struct FlappyJellyfish {
    pub owner: Entity,
}

pub fn spawn(
    owner: Entity,
    jellyfish_ent: Entity,
    flappy_meta_handle: Handle<FlappyJellyfishMeta>,
) -> StaticSystem<(), ()> {
    (move |mut entities: ResMut<Entities>,
           mut driving_jellyfishes: CompMut<DrivingJellyfish>,
           mut flappy_jellyfishes: CompMut<FlappyJellyfish>,
           mut fall_velocities: CompMut<FallVelocity>,
           assets: Res<AssetServer>,
           mut atlas_sprites: CompMut<AtlasSprite>,
           mut animated_sprites: CompMut<AnimatedSprite>,
           mut transforms: CompMut<Transform>| {
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
        let flappy_meta = assets.get(flappy_meta_handle);
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

pub fn kill(flappy: Entity) -> StaticSystem<(), ()> {
    (move |mut entities: ResMut<Entities>| {
        entities.kill(flappy);
        debug!("FLAPPY JELLYFISH | despawned");
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
