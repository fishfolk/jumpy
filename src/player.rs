use bevy::ecs::system::Command;
use bevy_tweening::Animator;

use crate::{
    item::{Item, ItemDropped, ItemGrabbed, ItemUsed},
    metadata::{GameMeta, PlayerMeta, Settings},
    networking::proto::ClientMatchInfo,
    physics::KinematicBody,
    platform::Storage,
    prelude::*,
};

use self::{input::PlayerInputs, state::PlayerState};

pub mod input;
pub mod state;

/// The maximum number of players we may have in the game. This may change in the future.
pub const MAX_PLAYERS: usize = 4;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.world.init_component::<PlayerKilled>();
        app.add_plugin(input::PlayerInputPlugin)
            .add_plugin(state::PlayerStatePlugin)
            .register_type::<PlayerIdx>()
            .register_type::<PlayerKilled>()
            .extend_rollback_plugin(|plugin| {
                plugin
                    .register_rollback_type::<PlayerIdx>()
                    .register_rollback_type::<PlayerState>()
                    .register_rollback_type::<PlayerKilled>()
            })
            .extend_rollback_schedule(|schedule| {
                schedule.add_system_to_stage(
                    RollbackStage::PreUpdate,
                    hydrate_players.run_if_resource_exists::<GameMeta>(),
                );
            });
    }
}

/// The player index, for example Player 1, Player 2, and so on.
#[derive(
    Component, Deref, DerefMut, Reflect, Default, Serialize, Deserialize, Copy, Clone, Debug,
)]
#[reflect(Default, Component)]
pub struct PlayerIdx(pub usize);

/// Marker component indicating that a player has been killed.
///
/// This usually means their death animation is playing, and they are about to be de-spawned.
#[derive(Component, Reflect, Default, Copy, Clone, Debug)]
#[reflect(Default, Component)]
pub struct PlayerKilled;

/// A [`Command`] to kill a player.
///
/// The command will perform any actions needed for the player kill sequence, including sending
/// network messages, etc.
///
/// > **Note:** This doesn't despawn the player, it just puts the player into it's death animation.
pub struct PlayerKillCommand {
    /// The player to kill
    pub player: Entity,
    /// An optional position for the kill event.
    ///
    /// If [`None`] the position will be automatically determined from the player's [`Transform`].
    ///
    /// It may need to be manually specified when killing remote players that don't have accurate
    /// local positions.
    pub position: Option<Vec3>,
    /// An optional velocity for the kill event, used when determining how a carried item will be
    /// dropped if the player was moving.
    ///
    /// If [`None`] the velocity will be automatically determined from the player's kinematic body.
    ///
    /// It may need to be manually specified when killing remote players that don't have local
    /// velocities.
    pub velocity: Option<Vec2>,
}

impl PlayerKillCommand {
    /// Create a command to kill `player`.
    pub fn new(player: Entity) -> Self {
        Self {
            player,
            position: None,
            velocity: None,
        }
    }
}

impl Command for PlayerKillCommand {
    fn write(self, world: &mut World) {
        let already_killed = world.get::<PlayerKilled>(self.player).is_some();
        if already_killed {
            // No need to kill him again.
            return;
        }

        // If the entity is a player
        if world.get::<PlayerIdx>(self.player).is_some() {
            let position = self.position.unwrap_or_else(|| {
                world
                    .get::<Transform>(self.player)
                    .expect("Player without kinematic body")
                    .translation
            });
            let velocity = self.velocity.unwrap_or_else(|| {
                world
                    .get::<KinematicBody>(self.player)
                    .expect("Player without kinematic body")
                    .velocity
            });

            // Drop any items player was carrying
            PlayerSetInventoryCommand {
                player: self.player,
                item: None,
                position: Some(position),
                velocity: Some(velocity),
            }
            .write(world);

            // Add the maker component
            world.entity_mut(self.player).insert(PlayerKilled);
        } else {
            warn!("Tried to kill non-player entity")
        }
    }
}

/// A [`Command`] to despawn a player.
///
/// This will send any required network messages.
///
/// > **Note:** This is different than the [`PlayerKillCommand`] in that it immediately removes the
/// > player from the world, while [`PlayerKillCommand`] will usually cause the player to enter the
/// > death animation.
/// >
/// > [`PlayerDespawnCommand`] is usually sent at the end of the player death animation.
pub struct PlayerDespawnCommand {
    player: Entity,
}

impl PlayerDespawnCommand {
    /// Create a new [`PlayerDespawnCommand`].
    pub fn new(player: Entity) -> Self {
        Self { player }
    }
}

impl Command for PlayerDespawnCommand {
    fn write(self, world: &mut World) {
        // If the entity is a player
        if world.get::<PlayerIdx>(self.player).is_some() {
            // Despawn the player entity
            despawn_with_children_recursive(world, self.player);
        } else {
            warn!("Tried to despawn non-player entity")
        }
    }
}

/// A [`Command`] to set the player's inventory.
///
/// The command will perform any actions needed including sending events, etc.
pub struct PlayerSetInventoryCommand {
    /// The player to set the inventory of.
    pub player: Entity,
    /// The item to put in the player inventory, or `None` if they should drop any existing item.
    pub item: Option<Entity>,

    /// An optional position for where to drop/grab the item.
    ///
    /// If `position` is [`None`] the position will be determined from the player's [`Transform`].
    ///
    /// This is useful when dropping an item for a remote, networked player, which we do not have a
    /// reliable position for locally.
    pub position: Option<Vec3>,
    /// An optional velocity to use when dropping the item.
    ///
    /// See `position` above for details.
    pub velocity: Option<Vec2>,
}

impl PlayerSetInventoryCommand {
    /// Conveniently create a new [`PlayerSetInventoryCommand`].
    pub fn new(player: Entity, item: Option<Entity>) -> Self {
        Self {
            player,
            item,
            position: None,
            velocity: None,
        }
    }
}

impl Command for PlayerSetInventoryCommand {
    fn write(self, world: &mut World) {
        let current_inventory = get_player_inventory(world, self.player);

        // If there was a previous item in the inventory, drop it
        if let Some(current_item) = current_inventory {
            // Add the drop marker
            world.entity_mut(current_item).insert(ItemDropped {
                player: self.player,
            });

            world
                .entity_mut(self.player)
                .remove_children(&[current_item]);
        }

        // If there is a new item in the inventory, add it
        if let Some(item) = self.item {
            // Add the grab marker
            world.entity_mut(item).insert(ItemGrabbed {
                player: self.player,
            });
            world.entity_mut(self.player).push_children(&[item]);
        }
    }
}

/// A [`Command`] to have the player use the item they carrying, if any.
///
/// This will generate an item use event if there is an item in the players inventory when the
/// command is flushed.
pub struct PlayerUseItemCommand {
    /// The player that is using their item.
    pub player: Entity,

    /// An optional position to use the item from.
    ///
    /// If [`None`] the position will be taken from the player's [`Transform`].
    ///
    /// Specifying a specific position is useful when triggering an item use for a remote, networked
    /// player for which we don't have a reliable position locally.
    pub position: Option<Vec3>,
    /// An optional item to use.
    ///
    /// If [`None`] the item will be taken from the player's inventory.
    ///
    /// Specifying a specific item is useful when triggering an item use for a remote, networked
    /// player for which our local status for their current item might not be accurate.
    pub item: Option<Entity>,
}

impl PlayerUseItemCommand {
    /// Create a command to have `player` use their item.
    pub fn new(player: Entity) -> Self {
        Self {
            player,
            position: None,
            item: None,
        }
    }
}

impl Command for PlayerUseItemCommand {
    fn write(self, world: &mut World) {
        let item = self
            .item
            .or_else(|| get_player_inventory(world, self.player));

        if let Some(item) = item {
            world.entity_mut(item).insert(ItemUsed {
                player: self.player,
            });
        } else {
            warn!("Tried to use item when not carrying one");
        }
    }
}

/// Helper function to get the inventory of the player, from the world
pub fn get_player_inventory(world: &mut World, entity: Entity) -> Option<Entity> {
    let mut item_ent = None;
    let mut items_query = world.query_filtered::<Entity, With<Item>>();
    if let Some(children) = world.get::<Children>(entity) {
        for child in children {
            if items_query.get(world, *child).is_ok() {
                if item_ent.is_none() {
                    item_ent = Some(*child);
                } else {
                    warn!("Multiple items in player inventory is not supported!");
                }
            }
        }
    }
    item_ent
}

/// System to take entities with only a [`PlayerIdx`] and a [`Transform`] and to add the remaining
/// components to them to make them complete players.
fn hydrate_players(
    mut commands: Commands,
    mut players: Query<(Entity, &PlayerIdx, &mut Transform), Without<PlayerState>>,
    mut storage: ResMut<Storage>,
    game: Res<GameMeta>,
    player_inputs: Res<PlayerInputs>,
    player_meta_assets: Res<Assets<PlayerMeta>>,
    client_match_info: Option<Res<ClientMatchInfo>>,
) {
    let settings = storage.get(Settings::STORAGE_KEY);
    let settings = settings.as_ref().unwrap_or(&game.default_settings);

    for (entity, player_idx, mut player_transform) in &mut players {
        // Mutate the player transform to trigger an update to it's global transform component. This
        // isn't normally necessary, but since the player may not start off with a GlobalTransform
        // it may be required.
        player_transform.set_changed();

        let input = &player_inputs.players[player_idx.0];
        let meta = player_meta_assets
            .get(&input.selected_player)
            .expect("Player meta not loaded");

        let (animation_bank, animation_bank_sprite) =
            meta.spritesheet.get_animation_bank_and_sprite();

        let mut entity_commands = commands.entity(entity);

        entity_commands
            .insert(Name::new(format!("Player {}", player_idx.0)))
            .insert(PlayerState::default())
            .insert(animation_bank)
            .insert(animation_bank_sprite)
            .insert(GlobalTransform::default())
            .insert_bundle(VisibilityBundle::default());

        let kinematic_body = KinematicBody {
            size: Vec2::new(32.0, 48.0), // FIXME: Don't hardcode! Load from player meta.
            has_mass: true,
            has_friction: true,
            gravity: 1.5,
            ..default()
        };
        let input_manager_for_player = |player_idx| InputManagerBundle {
            input_map: settings.player_controls.get_input_map(player_idx),
            ..default()
        };

        if let Some(match_info) = &client_match_info {
            // Only add physics and input bundle for non-remote players if we are a multiplayer client
            if match_info.player_idx == player_idx.0 {
                // If we are a client in a multiplayer game, use the first player's controls
                let player_input_idx = if client_match_info.is_some() {
                    0
                // Otherwise, use the corresponding player's controls
                } else {
                    player_idx.0
                };

                entity_commands
                    .insert(kinematic_body)
                    .insert_bundle(input_manager_for_player(player_input_idx));

            // For remote players we add an `Animator` that will be used to tween it's transform for
            // smoothing player movement.
            } else {
                entity_commands.insert(Animator::<Transform>::default());
            }

        // If this is a local game
        } else {
            entity_commands
                .insert(kinematic_body)
                .insert_bundle(input_manager_for_player(player_idx.0));
        }
    }
}
