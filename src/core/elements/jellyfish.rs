use bones_framework::asset::dashmap::mapref::one::MappedRef;

use crate::prelude::*;

use super::flappy_jellyfish::{self, FlappyJellyfishMeta, KillFlappyJellyfish};

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
    pub flappy_meta: Handle<FlappyJellyfishMeta>,
}

impl JellyfishMeta {
    pub fn get_flappy_meta_from_asset(
        asset: MappedRef<'_, Cid, LoadedAsset, SchemaBox>,
    ) -> Option<Handle<FlappyJellyfishMeta>> {
        asset
            .try_cast_ref::<JellyfishMeta>()
            .ok()
            .map(|m| m.flappy_meta)
    }
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
        .add_system_to_stage(CoreStage::PostUpdate, update_unused_jellyfishes)
        .add_system_to_stage(CoreStage::PostUpdate, update_driving_jellyfishes);
    flappy_jellyfish::session_plugin(session);
}

#[derive(Clone, Debug, Default, HasSchema)]
pub struct Jellyfish;

#[derive(Clone, Copy, Debug, Default, HasSchema)]
pub struct DrivingJellyfish {
    pub owner: Entity,
    pub flappy: Entity,
}

fn hydrate(
    game_meta: Root<GameMeta>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    mut entities: ResMutInit<Entities>,
    assets: Res<AssetServer>,
    mut jellyfishes: CompMut<Jellyfish>,
    mut atlas_sprites: CompMut<AtlasSprite>,
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
            jellyfishes.insert(entity, Jellyfish);
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

fn update_unused_jellyfishes(
    entities: Res<Entities>,
    jellyfishes: Comp<Jellyfish>,
    driving_jellyfishes: Comp<DrivingJellyfish>,
    mut items_used: CompMut<ItemUsed>,
    player_inventories: PlayerInventories,
    mut commands: Commands,
) {
    for (jellyfish_ent, _jellyfish) in entities.iter_with(&jellyfishes) {
        if driving_jellyfishes.contains(jellyfish_ent) {
            continue;
        }

        if items_used.contains(jellyfish_ent) {
            items_used.remove(jellyfish_ent);

            let Some(inventory) = player_inventories
                .iter()
                .find_map(|inv| inv.filter(|i| i.inventory == jellyfish_ent))
            else {
                continue;
            };
            let owner = inventory.player;

            debug!("JELLYFISH | mount");
            commands.add(flappy_jellyfish::spawn(owner, jellyfish_ent));
        }
    }
}

fn update_driving_jellyfishes(
    entities: Res<Entities>,
    driving_jellyfishes: Comp<DrivingJellyfish>,
    mut items_used: CompMut<ItemUsed>,
    mut kill_flappy: CompMut<KillFlappyJellyfish>,
) {
    for (jellyfish_ent, driving_jellyfish) in entities.iter_with(&driving_jellyfishes) {
        if items_used.contains(jellyfish_ent) {
            items_used.remove(jellyfish_ent);

            debug!("JELLYFISH | boom");
            kill_flappy.insert(driving_jellyfish.flappy, KillFlappyJellyfish);
        }
    }
}
