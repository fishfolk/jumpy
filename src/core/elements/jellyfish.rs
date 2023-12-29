use crate::prelude::*;

use super::flappy_jellyfish::{FlappyJellyfish, FlappyJellyfishMeta};

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("jellyfish"))]
#[repr(C)]
pub struct JellyfishMeta {
    pub atlas: Handle<Atlas>,
    pub body_size: Vec2,
    pub throw_velocity: f32,
    pub angular_velocity: f32,
    pub fin_anim: Ustr,
    pub grab_offset: Vec2,
    flappy_meta: Handle<FlappyJellyfishMeta>,
}

pub fn game_plugin(game: &mut Game) {
    JellyfishMeta::register_schema();
    FlappyJellyfishMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update);
}

#[derive(Clone, Debug, HasSchema, Default)]
pub struct Jellyfish {
    status: JellyfishStatus,
}

#[derive(Copy, Clone, Debug, Default)]
pub enum JellyfishStatus {
    #[default]
    Unmounted,
    Mounted {
        flappy: Entity,
    },
}

fn hydrate(
    game_meta: Root<GameMeta>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    mut entities: ResMutInit<Entities>,
    assets: Res<AssetServer>,
    mut jellyfishes: CompMut<Jellyfish>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut respawn_points: CompMut<DehydrateOutOfBounds>,
    mut items: CompMut<Item>,
    mut item_grabs: CompMut<ItemGrab>,
    mut item_throws: CompMut<ItemThrow>,
    mut transforms: CompMut<Transform>,
    mut bodies: CompMut<KinematicBody>,
    mut spawner_manager: SpawnerManager,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    let spawner_entities = entities
        .iter_with_bitset(&not_hydrated_bitset)
        .collect::<Vec<_>>();

    for spawner_ent in spawner_entities {
        let element_handle = *element_handles.get(spawner_ent).unwrap();
        let element_meta = assets.get(element_handle.0);
        let transform = *transforms.get(spawner_ent).unwrap();

        if let Ok(JellyfishMeta {
            atlas,
            body_size,
            throw_velocity,
            angular_velocity,
            fin_anim,
            grab_offset,
            ..
        }) = assets.get(element_meta.data).try_cast_ref()
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            hydrated.insert(entity, MapElementHydrated);
            jellyfishes.insert(entity, Jellyfish::default());
            items.insert(entity, Item);
            item_grabs.insert(
                entity,
                ItemGrab {
                    fin_anim: *fin_anim,
                    sync_animation: false,
                    grab_offset: *grab_offset,
                },
            );
            item_throws.insert(
                entity,
                ItemThrow::strength(*throw_velocity).with_spin(*angular_velocity),
            );
            element_handles.insert(entity, element_handle);
            atlas_sprites.insert(entity, AtlasSprite::new(*atlas));
            animated_sprites.insert(entity, default());
            respawn_points.insert(entity, DehydrateOutOfBounds(spawner_ent));
            transforms.insert(entity, transform);
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Rectangle { size: *body_size },
                    has_mass: true,
                    has_friction: true,
                    gravity: game_meta.core.physics.gravity,
                    ..default()
                },
            );
            spawner_manager.create_spawner(spawner_ent, vec![entity]);
        }
    }
}

fn update(
    entities: Res<Entities>,
    mut jellyfishes: CompMut<Jellyfish>,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,
    mut items_used: CompMut<ItemUsed>,
    player_inventories: PlayerInventories,
    mut commands: Commands,
) {
    for (entity, (jellyfish, element_handle)) in
        entities.iter_with((&mut jellyfishes, &element_handles))
    {
        let element_meta = assets.get(element_handle.0);
        let asset = assets.get(element_meta.data);
        let Ok(JellyfishMeta { flappy_meta, .. }) = asset.try_cast_ref() else {
            continue;
        };

        if let Some(inventory) = player_inventories
            .iter()
            .find_map(|inv| inv.filter(|i| i.inventory == entity))
        {
            let player = inventory.player;

            if items_used.get(entity).is_some() {
                items_used.remove(entity);
                if let JellyfishStatus::Unmounted = jellyfish.status {
                    debug!("JELLYFISH | mount");
                    let jellyfish_ent = entity;
                    let flappy_meta_handle = *flappy_meta;
                    commands.add(
                        move |mut entities: ResMut<Entities>,
                              mut jellyfishes: CompMut<Jellyfish>,
                              mut flappy_jellyfishes: CompMut<FlappyJellyfish>,
                              assets: Res<AssetServer>,
                              mut atlas_sprites: CompMut<AtlasSprite>,
                              mut animated_sprites: CompMut<AnimatedSprite>,
                              mut transforms: CompMut<Transform>| {
                            let Some(jellyfish) = jellyfishes.get_mut(jellyfish_ent) else {
                                return;
                            };
                            let flappy_ent = entities.create();
                            jellyfish.status = JellyfishStatus::Mounted { flappy: flappy_ent };
                            flappy_jellyfishes
                                .insert(flappy_ent, FlappyJellyfish { owner: player });
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
                            let mut transf = *transforms.get(player).unwrap();
                            transf.translation.y += 75.0;
                            transforms.insert(flappy_ent, transf);
                            debug!("FLAPPY JELLYFISH | spawned");
                        },
                    );
                } else if let JellyfishStatus::Mounted { flappy } = jellyfish.status {
                    debug!("JELLYFISH | boom");
                    jellyfish.status = JellyfishStatus::Unmounted;
                    commands.add(move |mut entities: ResMut<Entities>| {
                        entities.kill(flappy);
                        debug!("FLAPPY JELLYFISH | despawned");
                    });
                }
            }
        }
    }
}
