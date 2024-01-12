use crate::{core::player::idle, prelude::*};

use super::flappy_jellyfish::{self, FlappyJellyfishMeta};

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("jellyfish"))]
#[repr(C)]
pub struct JellyfishMeta {
    /// The atlas for the item image.
    pub atlas: Handle<Atlas>,
    /// The size of the item.
    pub body_size: Vec2,
    /// The velocity of the item when thrown.
    pub throw_velocity: f32,
    /// The angular velocity of the item when thrown.
    pub angular_velocity: f32,
    /// The animation frame a player's fin should use when holding the item.
    pub fin_anim: Ustr,
    /// The offset relative to the center of the player that is holding the
    /// item.
    pub grab_offset: Vec2,
    /// The maximum amount of uses of the item. Resets when dropped.
    pub max_ammo: u32,
    /// The metadata of a flappy jellyfish.
    pub flappy_meta: Handle<FlappyJellyfishMeta>,
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
        .add_system_to_stage(CoreStage::PreUpdate, dehydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update_player_driving);
    flappy_jellyfish::session_plugin(session);
}

/// A jellyfish item.
#[derive(Clone, Debug, Default, HasSchema)]
pub struct Jellyfish {
    pub ammo: u32,
    pub flappy: Option<Entity>,
}

impl Jellyfish {
    pub fn new(ammo: u32) -> Self {
        Self { ammo, flappy: None }
    }
}

/// A marker component for players to indicate that they are driving a flappy
/// jellyfish.
#[derive(Clone, Debug, Default, HasSchema)]
pub struct PlayerDrivingJellyfish {
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
    mut spawners: CompMut<DehydrateOutOfBounds>,
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
            max_ammo,
            ..
        }) = assets.get(element_meta.data).try_cast_ref()
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            spawners.insert(entity, DehydrateOutOfBounds(spawner_ent));
            hydrated.insert(entity, MapElementHydrated);
            jellyfishes.insert(entity, Jellyfish::new(*max_ammo));
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

/// A marker component for jellyfish items to indicate that it has been used and
/// should dehydrate. This will also remove and despawn the jellyfish from the
/// player's inventory.
#[derive(Clone, Debug, Default, HasSchema)]
pub struct DehydrateJellyfish {
    pub owner: Entity,
}

fn dehydrate(
    entities: Res<Entities>,
    dehydrate_jellyfish: Comp<DehydrateJellyfish>,
    spawners: Comp<DehydrateOutOfBounds>,
    mut player_driving: CompMut<PlayerDrivingJellyfish>,
    mut player_inventories: CompMut<Inventory>,
    mut commands: Commands,
    mut hydrated: CompMut<MapElementHydrated>,
) {
    for (jellyfish_ent, (dehydrate, spawner)) in
        entities.iter_with((&dehydrate_jellyfish, &spawners))
    {
        if player_inventories
            .get(dehydrate.owner)
            .and_then(|inv| inv.0)
            .filter(|item| *item == jellyfish_ent)
            .is_some()
        {
            player_driving.remove(dehydrate.owner);
            player_inventories.insert(dehydrate.owner, Inventory(None));
        }
        commands.add(move |mut entities: ResMut<Entities>| entities.kill(jellyfish_ent));
        hydrated.remove(**spawner);
    }
}

fn update_player_driving(
    entities: Res<Entities>,
    player_indexes: Comp<PlayerIdx>,
    player_inventories: Comp<Inventory>,
    player_states: Comp<PlayerState>,
    mut jellyfishes: CompMut<Jellyfish>,
    player_inputs: Res<MatchInputs>,
    mut player_driving: CompMut<PlayerDrivingJellyfish>,
    mut commands: Commands,
) {
    for (player_ent, (player_idx, player_inv, player_state)) in
        entities.iter_with((&player_indexes, &player_inventories, &player_states))
    {
        let Some(jellyfish_ent) = player_inv.0 else {
            continue;
        };
        let Some(jellyfish) = jellyfishes.get_mut(jellyfish_ent) else {
            continue;
        };

        let player_control = player_inputs.players[player_idx.0 as usize].control;

        if player_control.shoot_pressed {
            if player_state.current != *idle::ID || player_driving.contains(player_ent) {
                continue;
            }
            commands.add(flappy_jellyfish::spawn_or_take_control(
                player_ent,
                jellyfish_ent,
                jellyfish.flappy,
            ));
        } else {
            player_driving.remove(player_ent);
        }
    }
}
