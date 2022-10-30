use bevy_tweening::Animator;

use crate::{
    metadata::{GameMeta, PlayerMeta, Settings},
    networking::{
        proto::{
            game::{PlayerEvent, PlayerEventFromServer},
            ClientMatchInfo,
        },
        server::NetServer,
    },
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
        app.add_plugin(input::PlayerInputPlugin)
            .add_plugin(state::PlayerStatePlugin)
            .register_type::<PlayerIdx>()
            .add_system_to_stage(
                FixedUpdateStage::PreUpdate,
                hydrate_players.run_if_resource_exists::<GameMeta>(),
            );
    }
}

/// The player index, for example Player 1, Player 2, and so on
#[derive(Component, Deref, DerefMut, Reflect, Default, Serialize, Deserialize)]
#[reflect(Default, Component)]
pub struct PlayerIdx(pub usize);

/// The item the player is carrying, if any.
///
/// The item will be an empty string if the player isn't carrying anything.
///
/// The string represents the path to the asset that the player is carrying. It doesn't matter what
/// kind of asset, the scripts will determine what to do when you are carrying it.
///
/// Maybe this should be an untyped asset handle, or maybe a portable asset ID of some kind, but
/// that makes it more difficult to interact with from scripts, so for now we just use a basic
/// string.
#[derive(Component, Deref, DerefMut, Reflect, Default, Serialize, Deserialize)]
#[reflect(Default, Component)]
pub struct PlayerItem(pub String);

fn hydrate_players(
    mut commands: Commands,
    mut players: Query<(Entity, &PlayerIdx, &mut Transform), Without<PlayerState>>,
    mut storage: ResMut<Storage>,
    game: Res<GameMeta>,
    player_inputs: Res<PlayerInputs>,
    player_meta_assets: Res<Assets<PlayerMeta>>,
    server: Option<Res<NetServer>>,
    client_match_info: Option<Res<ClientMatchInfo>>,
) {
    let settings = storage.get(Settings::STORAGE_KEY);
    let settings = settings.as_ref().unwrap_or(&game.default_settings);

    for (entity, player_idx, mut player_transform) in &mut players {
        // Mutate the player transform to trigger an update to it's global transform component. This
        // isn't normally necessary, but since the player may not start off with a GlobalTransform
        // it may be required.
        player_transform.set_changed();

        // If we are the server, broadcast a spawn event for each hydrated player
        if let Some(server) = &server {
            server.broadcast_reliable(&PlayerEventFromServer {
                player_idx: player_idx.0.try_into().unwrap(),
                kind: PlayerEvent::SpawnPlayer(player_transform.translation),
            });
        }

        let input = &player_inputs.players[player_idx.0];
        let meta = if let Some(meta) = player_meta_assets.get(&input.selected_player) {
            meta
        } else {
            continue;
        };

        let (animation_bank, animation_bank_sprite) =
            meta.spritesheet.get_animation_bank_and_sprite();

        let mut entity_commands = commands.entity(entity);

        entity_commands
            .insert(Name::new(format!("Player {}", player_idx.0)))
            .insert(PlayerState::default())
            .insert(PlayerItem::default())
            .insert(meta.clone())
            .insert(animation_bank)
            .insert(animation_bank_sprite)
            .insert(GlobalTransform::default())
            .insert_bundle(VisibilityBundle::default());

        let kinematic_body = KinematicBody {
            size: Vec2::new(38.0, 48.0), // FIXME: Don't hardcode! Load from player meta.
            has_mass: true,
            has_friction: true,
            gravity: 1.0,
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
