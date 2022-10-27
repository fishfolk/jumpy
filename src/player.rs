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

fn hydrate_players(
    mut commands: Commands,
    players: Query<(Entity, &PlayerIdx, &Transform), Without<PlayerState>>,
    mut storage: ResMut<Storage>,
    game: Res<GameMeta>,
    player_inputs: Res<PlayerInputs>,
    player_meta_assets: Res<Assets<PlayerMeta>>,
    server: Option<Res<NetServer>>,
    client_match_info: Option<Res<ClientMatchInfo>>,
) {
    let settings = storage.get(Settings::STORAGE_KEY);
    let settings = settings.as_ref().unwrap_or(&game.default_settings);

    for (entity, player_idx, player_transform) in &players {
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
            .insert(meta.clone())
            .insert(animation_bank)
            .insert(animation_bank_sprite)
            .insert(GlobalTransform::default())
            .insert_bundle(VisibilityBundle::default());

        // Only add physics and input bundle for non-remote players if we are a multiplayer client
        if let Some(match_info) = &client_match_info {
            if match_info.player_idx == player_idx.0 {
                entity_commands
                    .insert(KinematicBody {
                        size: Vec2::new(38.0, 48.0), // FIXME: Don't hardcode! Load from player meta.
                        has_mass: true,
                        has_friction: true,
                        gravity: 1.0,
                        ..default()
                    })
                    .insert_bundle(InputManagerBundle {
                        input_map: settings.player_controls.get_input_map(player_idx.0),
                        ..default()
                    });
            }
        }
    }
}
