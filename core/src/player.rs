//! Player controller, states, and animation implementation.

use std::collections::VecDeque;

use crate::{
    item::ItemGrabbed,
    physics::KinematicBody,
    prelude::{player_spawner::PlayerSpawner, *},
    random::GlobalRng,
};

mod state;
use bones_lib::animation::AnimationBankSprite;
pub use state::*;
use turborand::GenCore;

const PLAYER_COLORS: [Color; 4] = [
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::rgb(1.0, 0.0, 1.0),
];

pub fn install(session: &mut CoreSession) {
    state::install(session);

    // Add other player systems
    session
        .stages
        .add_system_to_stage(CoreStage::First, hydrate_players)
        .add_system_to_stage(CoreStage::First, player_ai_system)
        .add_system_to_stage(CoreStage::PostUpdate, play_itemless_fin_animations)
        .add_system_to_stage(CoreStage::PostUpdate, player_facial_animations)
        .add_system_to_stage(CoreStage::Last, delete_dead_ai_swords)
        .add_system_to_stage(CoreStage::Last, update_player_layers);
}

/// The player index, for example Player 1, Player 2, and so on.
#[derive(Clone, TypeUlid, Deref, DerefMut)]
#[ulid = "01GP49B2AMTYB6W8DWKBRF27FT"]
pub struct PlayerIdx(pub usize);

/// Contains the entities of the extra player layers, such as the player face and fin.
#[derive(Clone, TypeUlid)]
#[ulid = "01GQQRZ4V5WSRJTA1VTA816Z9T"]
pub struct PlayerLayers {
    pub fin_anim: Key,
    pub fin_ent: Entity,
    pub fin_offset: Vec2,
    pub face_ent: Entity,
    pub face_anim: Key,
}

impl PlayerLayers {
    pub const FIN_Z_OFFSET: f32 = 0.5;
    pub const FACE_Z_OFFSET: f32 = 0.01;
}

/// A component representing the current emote state of a player.
#[derive(Clone, TypeUlid, Default)]
#[ulid = "01GR4Q7MJF132EFY1RZZWECJK0"]
enum EmoteState {
    /// The player is not emoting
    #[default]
    Neutral,
    /// The player is emoting.
    Emoting(Emote),
}

/// A component representing a region in which a player should emote in some way.
///
/// For example, a lit grenade could have a
#[derive(Clone, TypeUlid)]
#[ulid = "01GR2QVYXRX5V689C4EMBDS3AQ"]
pub struct EmoteRegion {
    /// Whether or not the player must be looking at the center of the region to emote.
    pub direction_sensitive: bool,
    /// The size of the emote region
    pub size: Vec2,
    /// The emote the player should make
    pub emote: Emote,
    /// Whether or not this emote region is active. This provides an easy way to disable a region
    /// temporarily, without having to remove the component.
    pub active: bool,
}

impl Default for EmoteRegion {
    fn default() -> Self {
        Self {
            direction_sensitive: true,
            size: Vec2::ZERO,
            emote: default(),
            active: true,
        }
    }
}

/// A kind of emote the player can make.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Emote {
    /// The player is alarmed!! Like a lit grenade was just thrown at them.
    #[default]
    Alarm,
}

impl Emote {
    pub fn animation_key(&self) -> Key {
        match self {
            Emote::Alarm => key!("emote_alarm"),
        }
    }
}

/// Marker component indicating that a player has been killed.
///
/// This usually means their death animation is playing, and they are about to be de-spawned.
#[derive(Clone, TypeUlid)]
#[ulid = "01GP49AK25A8S9G2GYNAVE4PTN"]
pub struct PlayerKilled {
    pub hit_from: Option<Vec2>,
}

/// Events that can be used to trigger player actions, such as killing, setting inventory, etc.
#[derive(Clone, Debug)]
pub struct PlayerCommand;

impl PlayerCommand {
    /// Kill a player.
    ///
    /// > **Note:** This doesn't despawn the player, it just puts the player into it's death animation.
    pub fn kill(player: Entity, hit_from: Option<Vec2>) -> System {
        (move |entities: Res<Entities>,
               mut players_killed: CompMut<PlayerKilled>,
               mut items_dropped: CompMut<ItemDropped>,
               mut inventories: CompMut<Inventory>,
               player_indexes: Comp<PlayerIdx>| {
            if players_killed.contains(player) {
                // No need to kill him again
                return;
            }

            let Some(idx) = player_indexes.get(player) else {
                    // Not a player, just ignore it.
                    warn!("Tried to kill non-player entity.");
                    return;
                };

            debug!("Killing player: {}", idx.0);

            // Drop any items the player was carrying
            let inventory = inventories.get(player).cloned().unwrap_or_default();
            if let Some(item) = inventory.0 {
                if entities.is_alive(item) {
                    items_dropped.insert(item, ItemDropped { player });
                }
            }

            // Update the inventory
            inventories.insert(player, Inventory(None));

            players_killed.insert(player, PlayerKilled { hit_from });
        })
        .system()
    }
    /// Despawn a player.
    ///
    /// > **Note:** This is different than the [`kill`][Self::kill] event in that it immediately
    /// > removes the player from the world, while [`kill`][Self::kill] will usually cause the
    /// > player to enter the death animation.
    /// >
    /// > [`despawn`][Self::despawn] is usually sent at the end of the player death animation.
    pub fn despawn(player: Entity) -> System {
        (move |mut entities: ResMut<Entities>,
               attachments: Comp<Attachment>,
               player_indexes: Comp<PlayerIdx>,
               player_layers: Comp<PlayerLayers>,
               player_spawners: Comp<PlayerSpawner>,
               mut spawner_manager: SpawnerManager| {
            if player_indexes.contains(player) {
                entities
                    .iter_with(&attachments)
                    .filter(|(_, attachment)| attachment.entity == player)
                    .map(|(entity, _)| entity)
                    .collect::<Vec<_>>()
                    .iter()
                    .for_each(|entity| {
                        entities.kill(*entity);
                    });
                let layers = player_layers.get(player).unwrap();
                entities.kill(layers.fin_ent);
                entities.kill(layers.face_ent);
                entities.kill(player);

                // remove player from all player spawners
                spawner_manager.remove_spawned_entity_from_grouped_spawner(
                    player,
                    &player_spawners,
                    &entities,
                );
            } else {
                warn!("Tried to despawn non-player entity.");
            }
        })
        .system()
    }
    /// Set the player's inventory
    pub fn set_inventory(player: Entity, item: Option<Entity>) -> System {
        (move |mut items_grabbed: CompMut<ItemGrabbed>,
               mut items_dropped: CompMut<ItemDropped>,
               mut inventories: CompMut<Inventory>| {
            let inventory = inventories.get(player).cloned().unwrap_or_default();

            // If there was a previous item, drop it
            if let Some(item) = inventory.0 {
                items_dropped.insert(item, ItemDropped { player });
            }

            // If there is a new item, grab it
            if let Some(item) = item {
                items_grabbed.insert(item, ItemGrabbed { player });
            }

            // Update the inventory
            inventories.insert(player, Inventory(item));
        })
        .system()
    }
    /// Have the player use the item they are carrying, if any.
    pub fn use_item(player: Entity) -> System {
        (move |mut items_used: CompMut<ItemUsed>, inventories: CompMut<Inventory>| {
            // If the player has an item
            if let Some(item) = inventories.get(player).and_then(|x| x.0) {
                // Use it
                items_used.insert(item, ItemUsed);
            }
        })
        .system()
    }
}

#[derive(Clone, Debug, TypeUlid)]
#[ulid = "01GQWND0P969BCZF5JET9MY944"]
pub struct AiPlayer {
    /// Tick timer that is used for AI pausing logic.
    tick: Timer,
    /// Indicates the player is taking pause for the given number of ticks.
    pausing: u32,
    /// Buffers planned AI movements
    movement_buffer: Option<VecDeque<PlayerControl>>,
    /// The player that the AI is targeting.
    target_player: Option<Entity>,
}

impl Default for AiPlayer {
    fn default() -> Self {
        Self {
            tick: Timer::from_seconds(0.5, TimerMode::Repeating),
            pausing: 0,
            movement_buffer: Default::default(),
            target_player: Default::default(),
        }
    }
}

#[derive(Debug, TypeUlid, Clone)]
#[ulid = "01GRA68NKYG6X7C5D0WNA5W1VX"]
pub struct PathfindingDebugLines {
    pub entities: Vec<Entity>,
}

impl FromWorld for PathfindingDebugLines {
    fn from_world(world: &mut World) -> Self {
        let entities = world
            .run_initialized_system(
                |mut entities: ResMut<Entities>, mut transforms: CompMut<Transform>| {
                    let ents = (0..MAX_PLAYERS)
                        .map(|_| {
                            let ent = entities.create();

                            transforms.insert(
                                ent,
                                Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
                            );

                            ent
                        })
                        .collect::<Vec<_>>();

                    Ok(ents)
                },
            )
            .unwrap();

        Self { entities }
    }
}

fn player_ai_system(
    entities: Res<Entities>,
    nav_graph: ResMut<NavGraph>,
    mut player_inputs: ResMut<PlayerInputs>,
    mut ai_players: CompMut<AiPlayer>,
    player_indexes: Comp<PlayerIdx>,
    map: Res<LoadedMap>,
    transforms: Comp<Transform>,
    pathfinding_debug_line: ResMut<PathfindingDebugLines>,
    mut paths: CompMut<Path2d>,
    bodies: Comp<KinematicBody>,
    debug_settings: Res<DebugSettings>,
    rng: Res<GlobalRng>,
    time: Res<Time>,
) {
    const SWORD_SWING_DIST: f32 = 10.0;
    const AI_SPEED_MULTIPLIER: f32 = 0.65;

    for (ai_ent, (player_idx, transform, ai_player)) in
        entities.iter_with((&player_indexes, &transforms, &mut ai_players))
    {
        // Tick the AI timer
        ai_player.tick.tick(time.delta());

        // If a tick has elapsed
        if ai_player.tick.just_finished() {
            // If the player isn't pausing, then there's a 40% chance
            if ai_player.pausing == 0 && rng.chance(0.4) {
                // That we will pause for a random number of ticks between 0 and 2
                ai_player.pausing = (rng.f32_normalized() * 2.0).round() as u32
            }

            // If the player is pausing
            if ai_player.pausing > 0 {
                // Subtract a tick from how long they should pause.
                ai_player.pausing -= 1;
            }
        }

        // If the player is pausing, don't have the AI move this frame.
        if ai_player.pausing > 0 {
            continue;
        }

        let target_transform = match ai_player.target_player {
            Some(target_player) if transforms.contains(target_player) => {
                transforms.get(target_player).unwrap()
            }
            _ => {
                let players = entities
                    .iter_with((&player_indexes, &transforms))
                    .filter(|(ent, _)| *ent != ai_ent)
                    .collect::<Vec<_>>();
                if players.is_empty() {
                    continue;
                }

                let (target_player, (_, transform)) = players[rng.gen_usize() % players.len()];

                ai_player.target_player = Some(target_player);
                transform
            }
        };
        let target_pos = target_transform.translation.truncate();
        let tile = (target_pos / map.tile_size).floor().as_ivec2();
        let target_node = NavNode(tile);
        let ai_pos = transform.translation.truncate();
        let tile = (ai_pos / map.tile_size).floor().as_ivec2();
        let current_node = NavNode(tile);

        // Complete any previous movement instructions if we are in the middle of any
        if let Some(movement_buffer) = &mut ai_player.movement_buffer {
            if let Some(control) = movement_buffer.pop_front() {
                player_inputs.players[player_idx.0].control = control;
                if movement_buffer.is_empty() {
                    ai_player.movement_buffer = None;
                }
                continue;
            }
        }

        let path = petgraph::algo::astar(
            nav_graph.as_ref(),
            current_node,
            |x| x == target_node,
            |(_, _, edge)| edge.distance,
            |_| 0.0,
        );

        if let Some((_cost, path)) = path {
            if debug_settings.show_pathfinding_lines {
                paths.insert(
                    pathfinding_debug_line.entities[player_idx.0],
                    Path2d {
                        points: path
                            .iter()
                            .map(|x| x.0.as_vec2() * map.tile_size + map.tile_size / 2.0)
                            .collect(),
                        thickness: 2.0,
                        color: PLAYER_COLORS[player_idx.0],
                        ..default()
                    },
                );
            }

            if let Some(&next_node) = path.get(1) {
                let edge = nav_graph.edge_weight(current_node, next_node).unwrap();
                let mut movement_buffer = edge.inputs.clone();
                let mut first_movement = movement_buffer.pop_front().unwrap();

                // Slow down the AI movement according to the fixed multiplier
                first_movement.move_direction *= vec2(AI_SPEED_MULTIPLIER, 1.0);

                // This is a hack to prevent us from getting stuck when we think we should be falling
                // straight down and we actually need to move off of the block we're half-standing on.
                //
                // If we aren't moving at all, just move in the direction of player 1
                if bodies.get(ai_ent).unwrap().velocity == Vec2::ZERO
                    && first_movement.move_direction == Vec2::ZERO
                {
                    let sign = (path.get(2).unwrap_or(&next_node).x as f32 * map.tile_size.x
                        - transform.translation.x)
                        .signum();
                    first_movement.move_direction.x = sign;
                }

                player_inputs.players[player_idx.0].control = first_movement;
                if !movement_buffer.is_empty() {
                    ai_player.movement_buffer = Some(movement_buffer)
                }
            }

            if (target_pos - ai_pos).length() < SWORD_SWING_DIST {
                player_inputs.players[player_idx.0]
                    .control
                    .shoot_just_pressed = true;
                player_inputs.players[player_idx.0].control.shoot_pressed = true;
            }
        } else if debug_settings.show_pathfinding_lines {
            let pos =
                current_node.0.as_vec2() * map.tile_size + map.tile_size / 2.0 - vec2(0.0, 4.0);
            paths.insert(
                pathfinding_debug_line.entities[player_idx.0],
                Path2d {
                    points: vec![pos, pos + vec2(0.0, 4.0)],
                    thickness: 8.0,
                    color: Color::RED,
                    ..default()
                },
            );
        }

        if !debug_settings.show_pathfinding_lines {
            paths.remove(pathfinding_debug_line.entities[player_idx.0]);
        }
    }
}

fn hydrate_players(
    mut commands: Commands,
    mut entities: ResMut<Entities>,
    player_inputs: Res<PlayerInputs>,
    player_indexes: Comp<PlayerIdx>,
    player_assets: BevyAssets<PlayerMeta>,
    mut player_states: CompMut<PlayerState>,
    mut inventories: CompMut<Inventory>,
    mut animation_bank_sprites: CompMut<AnimationBankSprite>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut kinematic_bodies: CompMut<KinematicBody>,
    mut player_layers: CompMut<PlayerLayers>,
    mut player_body_attachments: CompMut<PlayerBodyAttachment>,
    mut transforms: CompMut<Transform>,
    mut emote_states: CompMut<EmoteState>,
    mut ai_players: CompMut<AiPlayer>,
) {
    let mut not_hydrated_bitset = player_states.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(player_indexes.bitset());

    // Create entities that we will use for the extra player layers
    let mut new_entities = (0..(not_hydrated_bitset.bit_count() * 2))
        .map(|_| entities.create())
        .collect::<Vec<_>>()
        .into_iter();

    for player_entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let player_idx = player_indexes.get(player_entity).unwrap();
        let player_handle = &player_inputs.players[player_idx.0].selected_player;
        let is_ai = player_inputs.players[player_idx.0].is_ai;

        let Some(meta) = player_assets.get(&player_handle.get_bevy_handle()) else {
            continue;
        };

        let animation_bank_sprite = AnimationBankSprite {
            current: "idle".try_into().unwrap(),
            animations: meta.layers.body.animations.frames.clone(),
            last_animation: default(),
        };

        player_states.insert(player_entity, default());
        emote_states.insert(player_entity, default());
        animation_bank_sprites.insert(player_entity, animation_bank_sprite);
        inventories.insert(player_entity, default());

        atlas_sprites.insert(
            player_entity,
            AtlasSprite {
                atlas: meta.layers.body.atlas.clone(),
                ..default()
            },
        );
        kinematic_bodies.insert(
            player_entity,
            KinematicBody {
                shape: ColliderShape::Rectangle {
                    size: meta.body_size,
                },
                has_mass: true,
                has_friction: false,
                gravity: meta.gravity,
                ..default()
            },
        );

        // Spawn the player's fin and face
        let fin_entity = new_entities.next().unwrap();
        let face_entity = new_entities.next().unwrap();
        player_layers.insert(
            player_entity,
            PlayerLayers {
                fin_anim: key!("idle"),
                fin_ent: fin_entity,
                fin_offset: Vec2::ZERO,
                face_anim: key!("idle"),
                face_ent: face_entity,
            },
        );

        // Fin
        transforms.insert(fin_entity, default());
        atlas_sprites.insert(
            fin_entity,
            AtlasSprite {
                atlas: meta.layers.fin.atlas.clone(),
                ..default()
            },
        );
        animation_bank_sprites.insert(
            fin_entity,
            AnimationBankSprite {
                current: "idle".try_into().unwrap(),
                animations: meta.layers.fin.animations.clone(),
                last_animation: default(),
            },
        );
        player_body_attachments.insert(
            fin_entity,
            PlayerBodyAttachment {
                sync_color: true,
                sync_animation: false,
                player: player_entity,
                offset: meta.layers.fin.offset.extend(PlayerLayers::FIN_Z_OFFSET),
            },
        );

        // Face
        transforms.insert(face_entity, default());
        atlas_sprites.insert(
            face_entity,
            AtlasSprite {
                atlas: meta.layers.face.atlas.clone(),
                ..default()
            },
        );
        animation_bank_sprites.insert(
            face_entity,
            AnimationBankSprite {
                current: "idle".try_into().unwrap(),
                animations: meta.layers.face.animations.clone(),
                last_animation: default(),
            },
        );
        player_body_attachments.insert(
            face_entity,
            PlayerBodyAttachment {
                sync_color: true,
                sync_animation: false,
                player: player_entity,
                offset: meta.layers.face.offset.extend(PlayerLayers::FACE_Z_OFFSET),
            },
        );

        // Handle AI players
        if is_ai {
            ai_players.insert(player_entity, default());

            // Give the player a sword NOTE: It's not good that we're duplicating the sword hydrate
            // functionality here, and this is pretty hacky, but the AI as it stands is temporary
            // anyway, so it's fine for now.
            commands.add(
                move |mut entities: ResMut<Entities>,
                      mut swords: CompMut<sword::Sword>,
                      mut element_handles: CompMut<ElementHandle>,
                      element_assets: BevyAssets<ElementMeta>,
                      mut hydrated: CompMut<MapElementHydrated>,
                      mut bodies: CompMut<KinematicBody>,
                      mut atlas_sprites: CompMut<AtlasSprite>,
                      mut items: CompMut<Item>,
                      mut transforms: CompMut<Transform>,
                      game_meta: Res<CoreMetaArc>,
                      mut attachments: CompMut<PlayerBodyAttachment>,
                      mut inventories: CompMut<Inventory>| {
                    let element_handle = game_meta
                        .map_elements
                        .iter()
                        .find(|handle| {
                            matches!(
                                element_assets
                                    .get(&handle.get_bevy_handle())
                                    .unwrap()
                                    .builtin,
                                BuiltinElementKind::Sword { .. }
                            )
                        })
                        .unwrap();
                    let element_meta = element_assets
                        .get(&element_handle.get_bevy_handle())
                        .unwrap();
                    if let BuiltinElementKind::Sword {
                        atlas,
                        body_size,
                        can_rotate,
                        bounciness,
                        grab_offset,
                        ..
                    } = &element_meta.builtin
                    {
                        let sword_ent = entities.create();
                        inventories.insert(player_entity, Inventory(Some(sword_ent)));
                        items.insert(sword_ent, Item);
                        swords.insert(sword_ent, sword::Sword::default());
                        atlas_sprites.insert(sword_ent, AtlasSprite::new(atlas.clone()));
                        transforms.insert(sword_ent, default());
                        element_handles.insert(sword_ent, ElementHandle(element_handle.clone()));
                        hydrated.insert(sword_ent, MapElementHydrated);
                        bodies.insert(
                            sword_ent,
                            KinematicBody {
                                shape: ColliderShape::Rectangle { size: *body_size },
                                has_mass: true,
                                has_friction: true,
                                can_rotate: *can_rotate,
                                bounciness: *bounciness,
                                gravity: game_meta.physics.gravity,
                                ..default()
                            },
                        );
                        attachments.insert(
                            sword_ent,
                            PlayerBodyAttachment {
                                sync_color: false,
                                sync_animation: false,
                                player: player_entity,
                                offset: grab_offset.extend(1.0),
                            },
                        );
                    }
                },
            );
        }
    }
}

/// System that makes sure the swords held by AI are despawned when they are killed.
fn delete_dead_ai_swords(
    mut entities: ResMut<Entities>,
    ai_players: Comp<AiPlayer>,
    items: Comp<Item>,
    dropped: Comp<ItemDropped>,
) {
    let mut to_kill = Vec::new();
    for (ent, (_item, dropped)) in entities.iter_with((&items, &dropped)) {
        if ai_players.contains(dropped.player) {
            to_kill.push(ent);
        }
    }
    for entity in to_kill {
        entities.kill(entity);
    }
}

/// Animate the player's fins while
fn play_itemless_fin_animations(
    entities: Res<Entities>,
    mut player_layers: CompMut<PlayerLayers>,
    animation_bank_sprites: Comp<AnimationBankSprite>,
    player_indexes: Comp<PlayerIdx>,
    player_inventories: PlayerInventories,
) {
    for (_, (player_layers, player_idx, animation_bank)) in
        entities.iter_with((&mut player_layers, &player_indexes, &animation_bank_sprites))
    {
        let inventory = player_inventories[player_idx.0];

        if inventory.is_none() {
            player_layers.fin_anim = animation_bank.current;
        }
    }
}

fn player_facial_animations(
    entities: Res<Entities>,
    mut player_layers: CompMut<PlayerLayers>,
    emote_regions: Comp<EmoteRegion>,
    transforms: Comp<Transform>,
    atlas_sprites: Comp<AtlasSprite>,
    mut emote_states: CompMut<EmoteState>,
    players_killed: Comp<PlayerKilled>,
    animation_bank_sprites: Comp<AnimationBankSprite>,
) {
    for (player_ent, (player_layer, atlas_sprite, animation_bank, emote_state)) in entities
        .iter_with((
            &mut player_layers,
            &atlas_sprites,
            &animation_bank_sprites,
            &mut emote_states,
        ))
    {
        if players_killed.contains(player_ent) {
            *emote_state = EmoteState::Neutral;
            player_layer.face_anim = animation_bank.current;
            continue;
        }

        let player_pos = transforms.get(player_ent).unwrap().translation.truncate();

        let mut triggered_emote = None;
        for (_, (emote_region, transform)) in entities.iter_with((&emote_regions, &transforms)) {
            if !emote_region.active {
                continue;
            }
            let emote_pos = transform.translation.truncate();

            let direction = if atlas_sprite.flip_x { -1.0f32 } else { 1.0 };
            let is_facing_region = direction.signum() != (player_pos.x - emote_pos.x).signum();
            if !emote_region.direction_sensitive || is_facing_region {
                let emote_rect = Rect::new(
                    emote_pos.x,
                    emote_pos.y,
                    emote_region.size.x,
                    emote_region.size.y,
                );
                if emote_rect.contains(player_pos) {
                    triggered_emote = Some(emote_region.emote);
                }
            }
        }

        if let Some(new_emote) = triggered_emote {
            if let EmoteState::Emoting(already_emote) = emote_state {
                if new_emote != *already_emote {
                    player_layer.face_anim = new_emote.animation_key();
                    *emote_state = EmoteState::Emoting(new_emote);
                }
            } else {
                player_layer.face_anim = new_emote.animation_key();
                *emote_state = EmoteState::Emoting(new_emote);
            }
        } else {
            *emote_state = EmoteState::Neutral;
            player_layer.face_anim = animation_bank.current;
        }
    }
}

/// System that reads the [`PlayerLayers`] component and updates the animated sprite banks to match
/// the animations specified.
fn update_player_layers(
    entities: Res<Entities>,
    player_inputs: ResMut<PlayerInputs>,
    mut animation_bank_sprites: CompMut<AnimationBankSprite>,
    mut player_body_attachments: CompMut<PlayerBodyAttachment>,
    player_layers: Comp<PlayerLayers>,
    player_indexes: Comp<PlayerIdx>,
    player_assets: BevyAssets<PlayerMeta>,
) {
    for (_ent, (layers, player_idx)) in entities.iter_with((&player_layers, &player_indexes)) {
        let player_handle = &player_inputs.players[player_idx.0].selected_player;
        let Some(player_meta) = player_assets.get(&player_handle.get_bevy_handle()) else {
            continue;
        };

        let face_bank = animation_bank_sprites.get_mut(layers.face_ent).unwrap();
        face_bank.current = layers.face_anim;

        let base_fin_offset = player_meta.layers.fin.offset;
        let total_fin_offset = base_fin_offset + layers.fin_offset;
        let fin_attachment = player_body_attachments.get_mut(layers.fin_ent).unwrap();
        fin_attachment.offset.x = total_fin_offset.x;
        fin_attachment.offset.y = total_fin_offset.y;

        let fin_bank = animation_bank_sprites.get_mut(layers.fin_ent).unwrap();
        fin_bank.current = layers.fin_anim;
    }
}
