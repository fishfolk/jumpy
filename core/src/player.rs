use std::collections::VecDeque;

use crate::{item::ItemGrabbed, physics::KinematicBody, prelude::*};

mod state;
use bones_lib::animation::AnimationBankSprite;
pub use state::*;

pub fn install(session: &mut GameSession) {
    state::install(session);

    // Add other player systems
    session
        .stages
        .add_system_to_stage(CoreStage::First, hydrate_players)
        .add_system_to_stage(CoreStage::PostUpdate, handle_player_events)
        .add_system_to_stage(CoreStage::PostUpdate, play_itemless_fin_animations)
        .add_system_to_stage(CoreStage::PostUpdate, player_facial_animations)
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
    pub const FIN_Z_OFFSET: f32 = 2.0;
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
pub struct PlayerKilled;

/// Resource containing the player event queue.
#[derive(Clone, TypeUlid, Debug, Default)]
#[ulid = "01GP49AK25A8S9G2GYNAVE4PTN"]
pub struct PlayerEvents {
    pub queue: VecDeque<PlayerEvent>,
}

impl PlayerEvents {
    /// Send a player event.
    pub fn send(&mut self, event: PlayerEvent) {
        self.queue.push_back(event);
    }

    #[inline]
    pub fn set_inventory(&mut self, player: Entity, item: Option<Entity>) {
        self.queue
            .push_back(PlayerEvent::SetInventory { player, item })
    }

    #[inline]
    pub fn use_item(&mut self, player: Entity) {
        self.queue.push_back(PlayerEvent::UseItem { player })
    }

    #[inline]
    pub fn kill(&mut self, player: Entity) {
        self.queue.push_back(PlayerEvent::Kill { player })
    }

    #[inline]
    pub fn despawn(&mut self, player: Entity) {
        self.queue.push_back(PlayerEvent::Despawn { player })
    }
}

/// Events that can be used to trigger player actions, such as killing, setting inventory, etc.
#[derive(Clone, Debug)]
pub enum PlayerEvent {
    /// Kill a player.
    ///
    /// > **Note:** This doesn't despawn the player, it just puts the player into it's death animation.
    Kill { player: Entity },
    /// Despawn a player.
    ///
    /// > **Note:** This is different than the [`Kill`][Self::Kill] event in that it immediately
    /// > removes the player from the world, while [`Kill`][Self::Kill] will usually cause the
    /// > player to enter the death animation.
    /// >
    /// > [`Despawn`][Self::Despawn] is usually sent at the end of the player death animation.
    Despawn { player: Entity },
    /// Set the player's inventory
    SetInventory {
        player: Entity,
        item: Option<Entity>,
    },
    /// Have the player use the item they are carrying, if any.
    UseItem { player: Entity },
}

fn handle_player_events(
    mut entities: ResMut<Entities>,
    mut player_events: ResMut<PlayerEvents>,
    mut players_killed: CompMut<PlayerKilled>,
    mut items_grabbed: CompMut<ItemGrabbed>,
    mut items_dropped: CompMut<ItemDropped>,
    mut items_used: CompMut<ItemUsed>,
    mut inventories: CompMut<Inventory>,
    attachments: Comp<Attachment>,
    player_indexes: Comp<PlayerIdx>,
    player_layers: Comp<PlayerLayers>,
) {
    while let Some(event) = player_events.queue.pop_front() {
        match event {
            PlayerEvent::Kill { player } => {
                if players_killed.contains(player) {
                    // No need to kill him again
                    continue;
                }

                let Some(idx) = player_indexes.get(player) else {
                    // Not a player, just ignore it.
                    warn!("Tried to kill non-player entity.");
                    continue;
                };

                debug!("Killing player: {}", idx.0);

                // Drop any items the player was carrying
                player_events
                    .queue
                    .push_front(PlayerEvent::SetInventory { player, item: None });

                players_killed.insert(player, PlayerKilled);
            }
            PlayerEvent::Despawn { player } => {
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
                } else {
                    warn!("Tried to despawn non-player entity.");
                }
            }
            PlayerEvent::SetInventory { player, item } => {
                let inventory = inventories.get(player).cloned().unwrap_or_default();

                // If there was a previous item, drop it
                if let Some(item) = inventory.0 {
                    items_dropped.insert(item, ItemDropped { player });
                }

                // If there is a new item, grab it
                if let Some(item) = item {
                    items_grabbed.insert(item, ItemGrabbed);
                }

                // Update the inventory
                inventories.insert(player, Inventory(item));
            }
            PlayerEvent::UseItem { player } => {
                // If the player has an item
                if let Some(item) = inventories.get(player).and_then(|x| x.0) {
                    // Use it
                    items_used.insert(item, ItemUsed);
                }
            }
        }
    }
}

fn hydrate_players(
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
                player: player_entity,
                offset: meta.layers.fin.offset.extend(PlayerLayers::FIN_Z_OFFSET),
                sync_animation: false,
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
                player: player_entity,
                offset: meta.layers.face.offset.extend(0.01),
                sync_animation: false,
            },
        );
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
