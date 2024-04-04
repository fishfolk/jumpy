use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("machine_gun"))]
#[repr(C)]
pub struct MachineGunMeta {
    pub grab_offset: Vec2,
    pub fin_anim: Ustr,

    pub body_size: Vec2,
    pub bounciness: f32,
    pub can_rotate: bool,
    pub throw_velocity: f32,
    pub angular_velocity: f32,
    pub atlas: Handle<Atlas>,

    pub max_ammo: u32,
    pub cooldown: Duration,
    pub empty_cooldown: Duration,
    pub bullet_meta: Handle<BulletMeta>,
    pub bullet_spread: f32,
    pub kickback: f32,

    pub shoot_sound_volume: f64,
    pub empty_shoot_sound_volume: f64,
    pub shoot_sound: Handle<AudioSource>,
    pub empty_shoot_sound: Handle<AudioSource>,
}

pub fn game_plugin(game: &mut Game) {
    MachineGunMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update);
}

#[derive(Clone, Debug, HasSchema, Default)]
pub struct MachineGun {
    pub ammo: u32,
    pub cooldown: Timer,
    pub empty_cooldown: Timer,
    pub state: MachineGunState,
}

#[derive(Default, Clone, Copy, Debug)]
pub enum MachineGunState {
    #[default]
    Idle,
    Shooting,
}

fn hydrate(
    game_meta: Root<GameMeta>,
    mut entities: ResMutInit<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    assets: Res<AssetServer>,
    mut machine_guns: CompMut<MachineGun>,
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
        let element_handle = *element_handles.get(spawner_ent).unwrap();
        let element_meta = assets.get(element_handle.0);

        if let Ok(MachineGunMeta {
            atlas,
            fin_anim,
            grab_offset,
            max_ammo,
            body_size,
            can_rotate,
            bounciness,
            throw_velocity,
            angular_velocity,
            ..
        }) = assets.get(element_meta.data).try_cast_ref()
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            items.insert(entity, Item);
            item_throws.insert(
                entity,
                ItemThrow::strength(*throw_velocity)
                    .with_spin(*angular_velocity)
                    .with_system(machine_gun_drop(entity, *max_ammo)),
            );
            item_grabs.insert(
                entity,
                ItemGrab {
                    fin_anim: *fin_anim,
                    sync_animation: false,
                    grab_offset: *grab_offset,
                },
            );
            machine_guns.insert(
                entity,
                MachineGun {
                    ammo: *max_ammo,
                    cooldown: Timer::new(Duration::from_millis(0), TimerMode::Once),
                    empty_cooldown: Timer::new(Duration::from_millis(0), TimerMode::Once),
                    state: MachineGunState::Idle,
                },
            );
            atlas_sprites.insert(entity, AtlasSprite::new(*atlas));
            respawn_points.insert(entity, DehydrateOutOfBounds(spawner_ent));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle);
            hydrated.insert(entity, MapElementHydrated);
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Rectangle { size: *body_size },
                    has_mass: true,
                    has_friction: true,
                    can_rotate: *can_rotate,
                    bounciness: *bounciness,
                    gravity: game_meta.core.physics.gravity,
                    ..default()
                },
            );
            spawner_manager.create_spawner(spawner_ent, vec![entity])
        }
    }
}

fn update(
    entities: Res<Entities>,
    mut commands: Commands,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,

    mut machine_guns: CompMut<MachineGun>,
    transforms: CompMut<Transform>,
    mut sprites: CompMut<AtlasSprite>,
    mut audio_center: ResMut<AudioCenter>,

    player_inventories: PlayerInventories,
    mut items_used: CompMut<ItemUsed>,
    items_dropped: CompMut<ItemDropped>,
    time: Res<Time>,

    mut bodies: CompMut<KinematicBody>,
) {
    for (entity, (machine_gun, element_handle)) in
        entities.iter_with((&mut machine_guns, &element_handles))
    {
        let element_meta = assets.get(element_handle.0);

        let asset = assets.get(element_meta.data);
        let Ok(MachineGunMeta {
            max_ammo,
            cooldown,
            empty_cooldown,
            bullet_meta,
            bullet_spread,
            shoot_sound,
            empty_shoot_sound,
            shoot_sound_volume,
            empty_shoot_sound_volume,
            kickback,
            ..
        }) = asset.try_cast_ref()
        else {
            unreachable!();
        };

        machine_gun.cooldown.tick(time.delta());
        machine_gun.empty_cooldown.tick(time.delta());

        // If the item is being held
        if let Some(Inv { player, .. }) = player_inventories.find_item(entity) {
            let sprite = sprites.get_mut(entity).unwrap();

            // Reset machine gun animation
            if let MachineGunState::Idle = machine_gun.state {
                sprite.index = 0;
            }

            // If the item is being used
            let item_used = items_used.remove(entity).is_some();
            if item_used {
                if machine_gun.cooldown.finished() {
                    // Reset fire cooldown
                    machine_gun.cooldown = Timer::new(*cooldown, TimerMode::Once);

                    // Run machine gun animation
                    if let MachineGunState::Shooting = machine_gun.state {
                        if sprite.index == 2 {
                            sprite.index = 3;
                        } else if sprite.index == 3 {
                            sprite.index = 2;
                        }
                    }

                    // Empty
                    if machine_gun.ammo.eq(&0) {
                        // Reset machine gun state if out of ammo
                        machine_gun.state = MachineGunState::Idle;
                        if machine_gun.empty_cooldown.finished() {
                            audio_center.play_sound(*empty_shoot_sound, *empty_shoot_sound_volume);
                            machine_gun.empty_cooldown =
                                Timer::new(*empty_cooldown, TimerMode::Once);
                        }
                        continue;
                    }

                    if matches!(machine_gun.state, MachineGunState::Idle) {
                        machine_gun.state = MachineGunState::Shooting;
                        sprite.index = 2;
                    }

                    // Subtract ammo
                    machine_gun.ammo = machine_gun
                        .ammo
                        .saturating_sub(1)
                        .clamp(0, machine_gun.ammo);
                    audio_center.play_sound(*shoot_sound, *shoot_sound_volume);

                    let player_sprite = sprites.get_mut(player).unwrap();
                    let player_flip_x = player_sprite.flip_x;
                    let player_body = bodies.get_mut(player).unwrap();

                    //Set kickback
                    player_body.velocity.x = if player_flip_x { 1.0 } else { -1.0 } * kickback;

                    let mut shoot_animation_transform = *transforms.get(entity).unwrap();
                    shoot_animation_transform.translation.z += 1.0;
                    shoot_animation_transform.translation.y += 8.0;
                    shoot_animation_transform.translation.x +=
                        if player_sprite.flip_x { -30.0 } else { 30.0 };

                    let bullet_meta = *bullet_meta;
                    let bullet_spread = *bullet_spread;

                    commands.add(
                        move |rng: Res<GlobalRng>,
                              mut entities: ResMutInit<Entities>,
                              mut transforms: CompMut<Transform>,
                              mut bullets: CompMut<Bullet>,
                              mut bullet_handles: CompMut<BulletHandle>| {
                            // spawn bullet
                            {
                                let ent = entities.create();
                                bullets.insert(
                                    ent,
                                    Bullet {
                                        owner: player,
                                        direction: if player_flip_x {
                                            vec2(-1.0, (rng.f32() - 0.5) * bullet_spread)
                                        } else {
                                            vec2(1.0, (rng.f32() - 0.5) * bullet_spread)
                                        },
                                    },
                                );
                                transforms.insert(ent, shoot_animation_transform);
                                bullet_handles.insert(ent, BulletHandle(bullet_meta));
                            }
                        },
                    );
                }
            } else {
                match machine_gun.state {
                    MachineGunState::Idle => (),
                    MachineGunState::Shooting => {
                        machine_gun.state = MachineGunState::Idle;
                    }
                }
            }
        }

        // If the item was dropped
        if items_dropped.get(entity).is_some() {
            // reload gun
            machine_gun.ammo = *max_ammo;
        }
    }
}

fn machine_gun_drop(entity: Entity, max_ammo: u32) -> StaticSystem<(), ()> {
    (move |mut machine_guns: CompMut<MachineGun>| {
        machine_guns.get_mut(entity).unwrap().ammo = max_ammo;
    })
    .system()
}
