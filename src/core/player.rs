//! Player controller, states, and animation implementation.

use std::collections::VecDeque;

use crate::prelude::*;

mod state;
pub use state::*;
use turborand::GenCore;

const PLAYER_COLORS: [Color; 4] = [
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::rgb(1.0, 0.0, 1.0),
];

pub fn plugin(session: &mut Session) {
    session.install_plugin(state::plugin);

    // Add other player systems
    session
        .stages
        .add_system_to_stage(CoreStage::First, hydrate_players)
        .add_system_to_stage(CoreStage::First, player_ai_system)
        .add_system_to_stage(CoreStage::PostUpdate, play_itemless_fin_animations)
        .add_system_to_stage(CoreStage::PostUpdate, player_facial_animations)
        .add_system_to_stage(CoreStage::PostUpdate, equip_hats)
        .add_system_to_stage(CoreStage::Last, delete_dead_ai_swords)
        .add_system_to_stage(CoreStage::Last, update_player_layers);
}

/// The player index, for example Player 1, Player 2, and so on.
#[derive(Clone, HasSchema, Deref, DerefMut, Default)]
pub struct PlayerIdx(pub u32);

/// Contains the entities of the extra player layers, such as the player face and fin.
#[derive(Clone, HasSchema, Default)]
pub struct PlayerLayers {
    pub fin_anim: Ustr,
    pub fin_ent: Entity,
    pub fin_offset: Vec2,
    pub face_ent: Entity,
    pub face_anim: Ustr,
    pub hat_ent: Option<Entity>,
}

impl PlayerLayers {
    pub const FIN_Z_OFFSET: f32 = 0.5;
    pub const FACE_Z_OFFSET: f32 = 0.01;
    pub const HAT_Z_OFFSET: f32 = 0.02;
}

/// A component representing the current emote state of a player.
#[derive(Clone, HasSchema, Default)]
enum EmoteState {
    /// The player is not emoting
    #[default]
    Neutral,
    /// The player is emoting.
    Emoting((Emote, Entity)),
}

/// A component representing a region in which a player should emote in some way.
///
/// For example, a lit grenade could have a
#[derive(Debug, Clone, HasSchema)]
pub struct EmoteRegion {
    /// The entity that owns this emote region.
    pub owner: Option<Entity>,
    /// A buffer to prevent the player from spamming emotes.
    pub buffer: Option<Timer>,
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
            owner: None,
            active: true,
            buffer: None,
            emote: default(),
            size: Vec2::ZERO,
            direction_sensitive: true,
        }
    }
}

impl EmoteRegion {
    pub fn basic(emote: Emote, size: Vec2, sensitive: bool) -> Self {
        Self {
            emote,
            size,
            active: true,
            direction_sensitive: sensitive,
            ..default()
        }
    }
}

/// A kind of emote the player can make.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Emote {
    /// The player is alarmed!! Like a lit grenade was just thrown at them.
    #[default]
    Alarm,
}

impl Emote {
    pub fn face_animation_key(&self) -> Ustr {
        match self {
            Emote::Alarm => ustr("emote_alarm"),
        }
    }

    pub fn emote_animation_key(&self) -> Ustr {
        match self {
            Emote::Alarm => ustr("alarm"),
        }
    }
}

impl Emote {
    pub fn start_animation(player_ent: Entity, emote: Self) -> StaticSystem<(), ()> {
        (move |meta: Root<GameMeta>,
               assets: ResMut<AssetServer>,
               mut emote_states: CompMut<EmoteState>,
               mut entities: ResMut<Entities>,
               mut transforms: CompMut<Transform>,
               mut attachments: CompMut<PlayerBodyAttachment>,
               mut sprites: CompMut<AtlasSprite>,
               mut animated_sprites: CompMut<AnimatedSprite>| {
            let emote_key = emote.emote_animation_key();
            let Some(emote_meta) = meta
                .core
                .player_emotes
                .iter()
                .find(|(key, _)| **key == emote_key)
                .map(|(_, handle)| assets.get(*handle))
            else {
                return;
            };

            let ent = entities.create();

            let emote_state = emote_states.get_mut(player_ent).unwrap();
            *emote_state = EmoteState::Emoting((emote, ent));

            let transform = *transforms.get(player_ent).unwrap();
            transforms.insert(ent, transform);
            attachments.insert(
                ent,
                PlayerBodyAttachment {
                    player: player_ent,
                    offset: emote_meta.offset.extend(PlayerLayers::FACE_Z_OFFSET),
                    head: true,
                    ..default()
                },
            );
            sprites.insert(
                ent,
                AtlasSprite {
                    atlas: emote_meta.atlas,
                    ..default()
                },
            );
            animated_sprites.insert(
                ent,
                AnimatedSprite {
                    frames: emote_meta.animation.frames.clone(),
                    fps: emote_meta.animation.fps,
                    repeat: true,
                    ..default()
                },
            );
        })
        .system()
    }

    pub fn stop_animation(player_ent: Entity) -> StaticSystem<(), ()> {
        (move |mut emote_states: CompMut<EmoteState>, mut entities: ResMut<Entities>| {
            let emote_state = emote_states.get_mut(player_ent).unwrap();
            if let EmoteState::Emoting((_, emote_ent)) = *emote_state {
                entities.kill(emote_ent);
            }
            *emote_state = EmoteState::Neutral;
        })
        .system()
    }
}

/// Marker component indicating that a player has been killed.
///
/// This usually means their death animation is playing, and they are about to be de-spawned.
#[derive(Clone, HasSchema, Default)]
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
    pub fn kill(player: Entity, hit_from: Option<Vec2>) -> StaticSystem<(), ()> {
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
    pub fn despawn(player: Entity) -> StaticSystem<(), ()> {
        (move |mut entities: ResMutInit<Entities>,
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
    pub fn set_inventory(player: Entity, item: Option<Entity>) -> StaticSystem<(), ()> {
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
    pub fn use_item(player: Entity) -> StaticSystem<(), ()> {
        (move |mut items_used: CompMut<ItemUsed>, inventories: CompMut<Inventory>| {
            // If the player has an item
            if let Some(item) = inventories.get(player).and_then(|x| x.0) {
                // Use it
                items_used.insert(item, ItemUsed { owner: player });
            }
        })
        .system()
    }
}

#[derive(Clone, Debug, HasSchema)]
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

#[derive(Debug, HasSchema, Clone)]
#[schema(no_default)]
pub struct PathfindingDebugLines {
    pub entities: Vec<Entity>,
}

impl FromWorld for PathfindingDebugLines {
    fn from_world(world: &World) -> Self {
        let entities = world.run_system(
            |mut entities: ResMut<Entities>, mut transforms: CompMut<Transform>| {
                (0..MAX_PLAYERS)
                    .map(|_| {
                        let ent = entities.create();

                        transforms
                            .insert(ent, Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)));

                        ent
                    })
                    .collect::<Vec<_>>()
            },
            (),
        );

        Self { entities }
    }
}

fn player_ai_system(
    entities: Res<Entities>,
    nav_graph: ResMutInit<NavGraph>,
    mut player_inputs: ResMutInit<MatchInputs>,
    mut ai_players: CompMut<AiPlayer>,
    player_indexes: Comp<PlayerIdx>,
    map: Res<LoadedMap>,
    transforms: Comp<Transform>,
    pathfinding_debug_line: ResMutInit<PathfindingDebugLines>,
    mut paths: CompMut<Path2d>,
    bodies: Comp<KinematicBody>,
    debug_settings: ResInit<DebugSettings>,
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
                player_inputs.players[player_idx.0 as usize].control = control;
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
                    pathfinding_debug_line.entities[player_idx.0 as usize],
                    Path2d {
                        points: path
                            .iter()
                            .map(|x| x.0.as_vec2() * map.tile_size + map.tile_size / 2.0)
                            .collect(),
                        thickness: 2.0,
                        color: PLAYER_COLORS[player_idx.0 as usize],
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

                player_inputs.players[player_idx.0 as usize].control = first_movement;
                if !movement_buffer.is_empty() {
                    ai_player.movement_buffer = Some(movement_buffer)
                }
            }

            if (target_pos - ai_pos).length() < SWORD_SWING_DIST {
                player_inputs.players[player_idx.0 as usize]
                    .control
                    .shoot_just_pressed = true;
                player_inputs.players[player_idx.0 as usize]
                    .control
                    .shoot_pressed = true;
            }
        } else if debug_settings.show_pathfinding_lines {
            let pos =
                current_node.0.as_vec2() * map.tile_size + map.tile_size / 2.0 - vec2(0.0, 4.0);
            paths.insert(
                pathfinding_debug_line.entities[player_idx.0 as usize],
                Path2d {
                    points: vec![pos, pos + vec2(0.0, 4.0)],
                    thickness: 8.0,
                    color: Color::RED,
                    ..default()
                },
            );
        }

        if !debug_settings.show_pathfinding_lines {
            paths.remove(pathfinding_debug_line.entities[player_idx.0 as usize]);
        }
    }
}

/// Resource that tracks which players have already been spawned before.
///
/// This lets us handle re-spawns differently, like not spawning you with a hat on a re-spawn.
///
/// TODO: This is probably temporary and will be removed when we have the proper tournament flow
/// down.
#[derive(Debug, Clone, HasSchema, Default)]
struct PlayersHaveSpawned {
    /// For each player, whether they have spawned before.
    pub players: [bool; MAX_PLAYERS],
}

/// Marker component for a player hat.
#[derive(Debug, Clone, HasSchema, Default)]
struct Hat(Handle<HatMeta>);

fn hydrate_players(
    mut commands: Commands,
    mut entities: ResMutInit<Entities>,
    game_meta: Root<GameMeta>,
    player_inputs: Res<MatchInputs>,
    player_indexes: Comp<PlayerIdx>,
    assets: Res<AssetServer>,
    mut player_states: CompMut<PlayerState>,
    mut inventories: CompMut<Inventory>,
    mut animation_bank_sprites: CompMut<AnimationBankSprite>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut kinematic_bodies: CompMut<KinematicBody>,
    mut camera_subjects: CompMut<CameraSubject>,
    mut player_layers: CompMut<PlayerLayers>,
    mut player_body_attachments: CompMut<PlayerBodyAttachment>,
    mut transforms: CompMut<Transform>,
    mut emote_states: CompMut<EmoteState>,
    mut ai_players: CompMut<AiPlayer>,
    mut invincibles: CompMut<Invincibility>,
    mut element_kill_callbacks: CompMut<ElementKillCallback>,
    mut players_have_spawned: ResMutInit<PlayersHaveSpawned>,
    mut item_grabs: CompMut<ItemGrab>,
    mut item_throws: CompMut<ItemThrow>,
    mut items: CompMut<Item>,
    mut hats: CompMut<Hat>,
) {
    let mut not_hydrated_bitset = player_states.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(player_indexes.bitset());

    // Create entities that we will use for the extra player layers
    let entity_count = not_hydrated_bitset.bit_count() * 3;
    let mut new_entities = (0..entity_count)
        .map(|_| entities.create())
        .collect::<Vec<_>>()
        .into_iter();
    // If a player doesn't have a hat we'll kill the entity we allocated for it.
    let mut to_kill = Vec::with_capacity(entity_count / 3);

    for player_entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let player_idx = player_indexes.get(player_entity).unwrap();
        let player_has_spawned = &mut players_have_spawned.players[player_idx.0 as usize];
        let player_handle = player_inputs.players[player_idx.0 as usize].selected_player;
        let player_hat = &player_inputs.players[player_idx.0 as usize].selected_hat;
        let is_ai = player_inputs.players[player_idx.0 as usize].is_ai;

        let meta = assets.get(player_handle);

        let animation_bank_sprite = AnimationBankSprite {
            current: "idle".try_into().unwrap(),
            animations: meta.layers.body.animations.frames.clone(),
            last_animation: default(),
        };

        player_states.insert(player_entity, default());
        emote_states.insert(player_entity, default());
        animation_bank_sprites.insert(player_entity, animation_bank_sprite);
        inventories.insert(player_entity, default());
        invincibles.insert(
            player_entity,
            Invincibility::new(game_meta.core.config.respawn_invincibility_time),
        );
        element_kill_callbacks.insert(
            player_entity,
            ElementKillCallback::new(player_kill_callback(player_entity)),
        );

        atlas_sprites.insert(
            player_entity,
            AtlasSprite {
                atlas: meta.layers.body.atlas,
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
                is_controlled: true,
                ..default()
            },
        );
        camera_subjects.insert(player_entity, default());

        // Spawn the player's fin and face
        let fin_entity = new_entities.next().unwrap();
        let face_entity = new_entities.next().unwrap();

        // Fin
        transforms.insert(fin_entity, default());
        atlas_sprites.insert(
            fin_entity,
            AtlasSprite {
                atlas: meta.layers.fin.atlas,
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
                head: false,
                player: player_entity,
                offset: meta.layers.fin.offset.extend(PlayerLayers::FIN_Z_OFFSET),
            },
        );

        // Face
        transforms.insert(face_entity, default());
        atlas_sprites.insert(
            face_entity,
            AtlasSprite {
                atlas: meta.layers.face.atlas,
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
                player: player_entity,
                sync_color: true,
                sync_animation: false,
                head: true,
                offset: meta.layers.face.offset.extend(PlayerLayers::FACE_Z_OFFSET),
            },
        );

        // Hat
        let hat_ent = new_entities.next().unwrap();
        let hat_ent = if !*player_has_spawned {
            if let Some(hat_handle) = player_hat {
                let hat_meta = assets.get(*hat_handle);
                let atlas = hat_meta.atlas;
                let offset = hat_meta.offset.extend(PlayerLayers::HAT_Z_OFFSET);
                hats.insert(hat_ent, Hat(*hat_handle));
                transforms.insert(hat_ent, default());
                atlas_sprites.insert(hat_ent, AtlasSprite { atlas, ..default() });
                player_body_attachments.insert(
                    hat_ent,
                    PlayerBodyAttachment {
                        player: player_entity,
                        offset,
                        head: true,
                        sync_animation: false,
                        sync_color: true,
                    },
                );
                kinematic_bodies.insert(
                    hat_ent,
                    KinematicBody {
                        shape: ColliderShape::Rectangle {
                            size: hat_meta.body_size,
                        },
                        has_mass: true,
                        has_friction: true,
                        gravity: meta.gravity,
                        is_deactivated: true,
                        ..default()
                    },
                );
                items.insert(hat_ent, Item);
                item_grabs.insert(
                    hat_ent,
                    ItemGrab {
                        fin_anim: "grab_2".into(),
                        grab_offset: Vec2::ZERO,
                        sync_animation: false,
                    },
                );
                item_throws.insert(hat_ent, ItemThrow::strength(7.0));
                Some(hat_ent)
            } else {
                to_kill.push(hat_ent);
                None
            }
        } else {
            to_kill.push(hat_ent);
            None
        };

        // Insert player layers component
        player_layers.insert(
            player_entity,
            PlayerLayers {
                fin_anim: "idle".into(),
                fin_ent: fin_entity,
                fin_offset: Vec2::ZERO,
                face_anim: "idle".into(),
                face_ent: face_entity,
                hat_ent,
            },
        );

        *player_has_spawned = true;

        // Handle AI players
        if is_ai {
            ai_players.insert(player_entity, default());

            // Give the player a sword NOTE: It's not good that we're duplicating the sword hydrate
            // functionality here, and this is pretty hacky, but the AI as it stands is temporary
            // anyway, so it's fine for now.
            commands.add(
                move |mut entities: ResMutInit<Entities>,
                      mut swords: CompMut<sword::Sword>,
                      mut element_handles: CompMut<ElementHandle>,
                      assets: Res<AssetServer>,
                      mut hydrated: CompMut<MapElementHydrated>,
                      mut bodies: CompMut<KinematicBody>,
                      mut atlas_sprites: CompMut<AtlasSprite>,
                      mut items: CompMut<Item>,
                      mut transforms: CompMut<Transform>,
                      game_meta: Root<GameMeta>,
                      mut attachments: CompMut<PlayerBodyAttachment>,
                      mut inventories: CompMut<Inventory>| {
                    let element_handle = game_meta
                        .core
                        .map_elements
                        .iter()
                        .find(|handle| {
                            assets
                                .get(assets.get(**handle).data)
                                .try_cast_ref::<SwordMeta>()
                                .is_ok()
                        })
                        .unwrap();
                    let element_meta = assets.get(*element_handle);
                    if let Ok(SwordMeta {
                        atlas,
                        body_size,
                        can_rotate,
                        bounciness,
                        grab_offset,
                        ..
                    }) = assets.get(element_meta.data).try_cast_ref()
                    {
                        let sword_ent = entities.create();
                        inventories.insert(player_entity, Inventory(Some(sword_ent)));
                        items.insert(sword_ent, Item);
                        swords.insert(sword_ent, sword::Sword::default());
                        atlas_sprites.insert(sword_ent, AtlasSprite::new(*atlas));
                        transforms.insert(sword_ent, default());
                        element_handles.insert(sword_ent, ElementHandle(*element_handle));
                        hydrated.insert(sword_ent, MapElementHydrated);
                        bodies.insert(
                            sword_ent,
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
                        attachments.insert(
                            sword_ent,
                            PlayerBodyAttachment {
                                sync_color: false,
                                sync_animation: false,
                                player: player_entity,
                                head: false,
                                offset: grab_offset.extend(1.0),
                            },
                        );
                    }
                },
            );
        }
    }

    for entity in to_kill {
        entities.kill(entity);
    }
}

fn player_kill_callback(player_entity: Entity) -> StaticSystem<(), ()> {
    (move |mut entities: ResMutInit<Entities>,
           attachments: Comp<Attachment>,
           player_layers: Comp<PlayerLayers>| {
        entities
            .iter_with(&attachments)
            .filter(|(_, attachment)| attachment.entity == player_entity)
            .map(|(entity, _)| entity)
            .collect::<Vec<_>>()
            .iter()
            .for_each(|entity| {
                entities.kill(*entity);
            });
        let layers = player_layers.get(player_entity).unwrap();
        entities.kill(layers.fin_ent);
        entities.kill(layers.face_ent);
        entities.kill(player_entity);
    })
    .system()
}

/// System that makes sure the swords held by AI are despawned when they are killed.
fn delete_dead_ai_swords(
    mut entities: ResMutInit<Entities>,
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
        let inventory = player_inventories[player_idx.0 as usize];

        if inventory.is_none() {
            player_layers.fin_anim = animation_bank.current;
        }
    }
}

fn player_facial_animations(
    time: Res<Time>,
    entities: Res<Entities>,
    mut player_layers: CompMut<PlayerLayers>,
    mut emote_regions: CompMut<EmoteRegion>,
    transforms: Comp<Transform>,
    atlas_sprites: Comp<AtlasSprite>,
    mut emote_states: CompMut<EmoteState>,
    mut commands: Commands,
    players_killed: Comp<PlayerKilled>,
    animation_bank_sprites: CompMut<AnimationBankSprite>,
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
        for (_, (emote_region, transform)) in entities.iter_with((&mut emote_regions, &transforms))
        {
            if !emote_region.active {
                continue;
            }

            // If a buffer exists, tick it.
            if let Some(buffer) = emote_region.buffer.as_mut() {
                buffer.tick(time.delta());

                // If the buffer is not finished, and the player is the owner, skip this emote.
                if let Some(owner) = emote_region.owner.as_ref() {
                    if *owner == player_ent && !(buffer.finished()) {
                        continue;
                    }
                }
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
                if new_emote != already_emote.0 {
                    player_layer.face_anim = new_emote.face_animation_key();
                    commands.add(Emote::stop_animation(player_ent));
                    commands.add(Emote::start_animation(player_ent, new_emote));
                }
            } else {
                player_layer.face_anim = new_emote.face_animation_key();
                commands.add(Emote::start_animation(player_ent, new_emote));
            }
        } else {
            player_layer.face_anim = animation_bank.current;
            commands.add(Emote::stop_animation(player_ent));
        }
    }
}

/// System that reads the [`PlayerLayers`] component and updates the animated sprite banks to match
/// the animations specified.
fn update_player_layers(
    entities: Res<Entities>,
    player_inputs: ResMutInit<MatchInputs>,
    mut animation_bank_sprites: CompMut<AnimationBankSprite>,
    mut player_body_attachments: CompMut<PlayerBodyAttachment>,
    player_layers: Comp<PlayerLayers>,
    player_indexes: Comp<PlayerIdx>,
    assets: Res<AssetServer>,
) {
    for (_ent, (layers, player_idx)) in entities.iter_with((&player_layers, &player_indexes)) {
        let player_handle = player_inputs.players[player_idx.0 as usize].selected_player;
        let player_meta = assets.get(player_handle);

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

/// Equip player hats that have been picked up and used.
fn equip_hats(
    entities: Res<Entities>,
    mut items_used: CompMut<ItemUsed>,
    mut kinematic_bodies: CompMut<KinematicBody>,
    mut player_body_attachments: CompMut<PlayerBodyAttachment>,
    hats: Comp<Hat>,
    assets: Res<AssetServer>,
    player_inventories: PlayerInventories,
    mut player_layers: CompMut<PlayerLayers>,
    mut inventories: CompMut<Inventory>,
) {
    for (hat_ent, hat) in entities.iter_with(&hats) {
        // If the hat is being held
        if let Some(Inv { player, .. }) = player_inventories.find_item(hat_ent) {
            if items_used.remove(hat_ent).is_some() {
                inventories.get_mut(player).unwrap().0 = None;

                let hat_meta = assets.get(hat.0);
                kinematic_bodies.get_mut(hat_ent).unwrap().is_deactivated = true;
                player_body_attachments.insert(
                    hat_ent,
                    PlayerBodyAttachment {
                        player,
                        offset: hat_meta.offset.extend(PlayerLayers::HAT_Z_OFFSET),
                        head: true,
                        sync_animation: false,
                        sync_color: true,
                    },
                );
                player_layers.get_mut(player).unwrap().hat_ent = Some(hat_ent);
            }
        }
    }
}
