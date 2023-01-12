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
        .add_system_to_stage(CoreStage::PostUpdate, handle_player_events)
        .add_system_to_stage(CoreStage::First, hydrate_players);
}

/// The player index, for example Player 1, Player 2, and so on.
#[derive(Clone, TypeUlid)]
#[ulid = "01GP49B2AMTYB6W8DWKBRF27FT"]
pub struct PlayerIdx(pub usize);

/// An intventory component, indicating that entiy
#[derive(Clone, TypeUlid, Default, Deref, DerefMut)]
#[ulid = "01GP4D6M2QBSKZMEZMM22YGG41"]
pub struct Inventory(pub Option<Entity>);

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
    player_indexes: Comp<PlayerIdx>,
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
    entities: Res<Entities>,
    player_inputs: Res<PlayerInputs>,
    player_indexes: Comp<PlayerIdx>,
    player_assets: BevyAssets<PlayerMeta>,
    mut player_states: CompMut<PlayerState>,
    mut inventories: CompMut<Inventory>,
    mut animation_bank_sprites: CompMut<AnimationBankSprite>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut kinematic_bodies: CompMut<KinematicBody>,
) {
    let mut not_hydrated_bitset = player_states.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(player_indexes.bitset());

    for entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let player_idx = player_indexes.get(entity).unwrap();
        let player_handle = &player_inputs.players[player_idx.0].selected_player;

        let Some(meta) = player_assets.get(&player_handle.get_bevy_handle()) else {
            continue;
        };

        let animation_bank_sprite = AnimationBankSprite {
            current: "idle".try_into().unwrap(),
            animations: meta.animations.clone(),
            last_animation: default(),
        };

        player_states.insert(entity, default());
        animation_bank_sprites.insert(entity, animation_bank_sprite);
        inventories.insert(entity, default());
        atlas_sprites.insert(
            entity,
            AtlasSprite {
                atlas: meta.atlas.clone(),
                ..default()
            },
        );
        kinematic_bodies.insert(
            entity,
            KinematicBody {
                size: meta.body_size,
                has_mass: true,
                has_friction: true,
                gravity: meta.gravity,
                ..default()
            },
        );
    }
}
