#[allow(unused_imports)]
use std::ops::Deref;

#[cfg(not(target_arch = "wasm32"))]
use bones_framework::networking::{socket::Socket, NetworkSocket, SocketTarget, SyncingInfo};

use crate::prelude::*;

use super::player_image::player_image;

pub fn session_plugin(session: &mut Session) {
    session.add_system_to_stage(Update, scoring_menu_system);
}

pub fn game_plugin(game: &mut Game) {
    game.insert_shared_resource(ScoringMenuState::default());
}

#[derive(HasSchema, Debug, Default, Clone)]
pub struct ScoringMenuState {
    pub active: bool,
    pub ready_players: HashSet<PlayerIdx>,
    pub match_score: MatchScore,
    pub next_maps: Option<MapPool>,
}

impl ScoringMenuState {
    /// Reset menu state
    pub fn reset(&mut self) {
        *self = default();
    }
}

struct PlayerScoreInfo {
    pub entity: Entity,
    pub player_idx: PlayerIdx,
    pub score: u32,
}

const SCORING_MESSAGE_MAGIC: u8 = 183;

#[derive(Serialize, Deserialize)]
struct ScoringMessage {
    pub data: ScoringMessageEnum,
    pub magic: u8,
}

#[derive(Serialize, Deserialize)]
enum ScoringMessageEnum {
    PlayerReady(u32),
}

impl From<ScoringMessageEnum> for ScoringMessage {
    fn from(value: ScoringMessageEnum) -> Self {
        Self {
            data: value,
            magic: SCORING_MESSAGE_MAGIC,
        }
    }
}

/// The width of each player panel
const PLAYER_PANEL_WIDTH: f32 = 200.0;

/// If Player won the match or is tied for win
enum PlayerWon {
    Tied,
    SoleWinner,
}

fn scoring_menu_system(
    meta: Root<GameMeta>,
    mut sessions: ResMut<Sessions>,
    ctx: Res<EguiCtx>,
    mut state: ResMut<ScoringMenuState>,
    controls: Res<GlobalPlayerControls>,
    world: &World,
) {
    if !state.active {
        // Make sure state is cleared if menu is closed
        state.reset();
        return;
    }

    let mut continue_game = false;
    let mut game_won = false;
    if let Some(session) = sessions.get_mut(SessionNames::GAME) {
        let player_indices = session.world.components.get::<PlayerIdx>();
        let player_indices_ref = player_indices.borrow();
        let game_entities = session.world.get_resource::<Entities>().unwrap();
        let match_inputs = session.world.get_resource::<MatchInputs>().unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        let network_socket: Option<Socket> = session
            .world
            .get_resource::<SyncingInfo>()
            .and_then(|x| x.socket().cloned());

        // Build Vec<PlayerScoreInfo> sorted by player indices
        let mut player_entities: Vec<(Entity, &PlayerIdx)> =
            game_entities.iter_with(&player_indices_ref).collect();
        player_entities.sort_by_key(|x| x.1 .0);
        let player_score_info: Vec<PlayerScoreInfo> = player_entities
            .iter()
            .map(|x| PlayerScoreInfo {
                entity: x.0,
                player_idx: *x.1,
                score: state.match_score.score(*x.1),
            })
            .collect();

        // Contains any players who have broken win threshold. If multiple players have winning scores,
        // contains player with highest score. If players are tied, tied players are all included.
        let mut winning_players = Vec::<PlayerIdx>::default();

        let win_threshold = meta.core.config.winning_score_threshold;
        let mut highest_score = 0;

        for score_info in player_score_info.iter() {
            let score = score_info.score;
            if score >= win_threshold && score >= highest_score {
                if score > highest_score {
                    // New winning player, clear prev winners
                    winning_players.clear();
                    highest_score = score;
                }

                winning_players.push(score_info.player_idx);
            }
        }
        if winning_players.len() == 1 {
            game_won = true;
        }

        // Check for inputs from local players toggling ready state
        for (_, player_idx) in player_entities.iter() {
            if let Some(source) = match_inputs.get_control_source(player_idx.0 as usize) {
                if let Some(control) = controls.get(&source) {
                    if control.menu_confirm_just_pressed
                        && !state.ready_players.contains(*player_idx)
                    {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if let Some(socket) = network_socket.as_ref() {
                                socket.send_reliable(
                                    SocketTarget::All,
                                    &postcard::to_allocvec(&ScoringMessage::from(
                                        ScoringMessageEnum::PlayerReady(player_idx.0),
                                    ))
                                    .unwrap(),
                                );

                                debug!("Send message local player {} ready", player_idx.0);
                            }
                        }

                        state.ready_players.insert(**player_idx);
                    }
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(socket) = network_socket.as_ref() {
            handle_scoring_messages(socket, &mut state);
        }

        // Check if all non-ai players are ready
        let mut all_players_ready = true;
        for (_, player_idx) in player_entities.iter() {
            let is_ai = match_inputs
                .players
                .get(player_idx.0 as usize)
                .unwrap()
                .is_ai;
            if !is_ai && !state.ready_players.contains(*player_idx) {
                all_players_ready = false;
            }
        }
        if all_players_ready {
            continue_game = true;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(&ctx, |ui| {
                let screen_rect = ui.max_rect();

                // Determine x margin based on size of player panels, some extra spacing, and screen space available.
                // This way window does not cover whole screen if < 4 players, and has consistent layout.
                let player_panel_spacing = 20.0;
                let max_width =
                    (PLAYER_PANEL_WIDTH + player_panel_spacing) * player_score_info.len() as f32;
                let x_margin = (screen_rect.width() - max_width) / 2.0;

                // x margin is dynamic, y margin percentage of screen
                let outer_margin =
                    egui::style::Margin::symmetric(x_margin, screen_rect.height() * 0.2);

                ui.vertical_centered(|ui| {
                    BorderedFrame::new(&meta.theme.panel.border)
                        .margin(outer_margin)
                        .padding(meta.theme.panel.padding)
                        .show(ui, |ui| {
                            world.run_system(
                                scoring_menu,
                                (
                                    ui,
                                    &player_score_info,
                                    &*match_inputs,
                                    &state,
                                    &winning_players,
                                ),
                            );
                        });
                });
            });
    } else {
        error!("Scoring menu failed to load score from existing game session. Closing scoring UI.");
        state.reset();
    }

    if continue_game {
        state.reset();
        let next_maps = state.next_maps.clone();
        let reset_score = game_won;
        sessions.add_command(Box::new(move |sessions: &mut Sessions| {
            sessions.restart_game(next_maps, reset_score);
        }));
    }
}

fn scoring_menu(
    mut param: In<(
        &mut egui::Ui,
        &Vec<PlayerScoreInfo>,
        &MatchInputs,
        &ScoringMenuState,
        &Vec<PlayerIdx>,
    )>,
    meta: Root<GameMeta>,
    localization: Localization<GameMeta>,
    world: &World,
) {
    let (ui, player_score_info, match_inputs, menu_state, winning_players) = &mut *param;

    // Scoring heading label
    ui.vertical_centered(|ui| {
        let (text, color) = match winning_players.len() {
            0 => ("intermission", meta.theme.panel.font_color),
            1 => ("match-complete", meta.theme.colors.positive),
            _ => ("tied-for-win", meta.theme.panel.font_color),
        };

        ui.label(
            meta.theme
                .font_styles
                .heading
                .rich(localization.get(text))
                .color(color),
        );
    });

    ui.vertical_centered(|ui| {
        ui.horizontal_centered(|ui| {
            let player_count = player_score_info.len();
            let available_spacing = ui.available_width() - PLAYER_PANEL_WIDTH * player_count as f32;

            // Compute how much space to use in gaps between panels
            let spacing = available_spacing / (player_count + 1) as f32;
            ui.add_space(spacing);
            for (i, _) in player_score_info.iter().enumerate() {
                let player_input = match_inputs.players.get(i).unwrap();
                let player_score_info = player_score_info.get(i).unwrap();
                let player_idx = player_score_info.player_idx;

                let ready = menu_state.ready_players.contains(&player_idx);
                let player_won =
                    if winning_players.contains(&player_idx) && winning_players.len() == 1 {
                        Some(PlayerWon::SoleWinner)
                    } else if winning_players.contains(&player_idx) {
                        Some(PlayerWon::Tied)
                    } else {
                        None
                    };

                world.run_system(
                    player_score_panel,
                    (ui, player_input, player_score_info, ready, player_won),
                );
                ui.add_space(spacing);
            }
        });
    });

    let match_complete = winning_players.len() == 1;
    ui.horizontal(|ui| {
        if match_complete {
            ui.label(
                meta.theme
                    .font_styles
                    .normal
                    .rich(localization.get("press-confirm-to-play-again"))
                    .color(meta.theme.panel.font_color),
            );
        } else {
            ui.label(
                meta.theme
                    .font_styles
                    .normal
                    .rich(localization.get("press-confirm-to-ready-up"))
                    .color(meta.theme.panel.font_color),
            );
        }
    });
}

fn player_score_panel(
    mut params: In<(
        &mut egui::Ui,
        &PlayerInput,
        &PlayerScoreInfo,
        bool,
        Option<PlayerWon>,
    )>,
    meta: Root<GameMeta>,
    assets: Res<AssetServer>,
    localization: Localization<GameMeta>,
    world: &World,
) {
    let (ui, player_input, player_score_info, ready, won) = &mut *params;
    let panel = &meta.theme.panel;

    BorderedFrame::new(&panel.border)
        .padding(panel.padding)
        .show(ui, |ui| {
            // PLAYER_PANEL_WIDTH is total space for entire bordered frame, remove space lost to padding
            // and use this for inner contents.
            ui.set_width(PLAYER_PANEL_WIDTH - panel.padding.left - panel.padding.right);

            ui.vertical_centered(|ui| {
                ui.label(
                    meta.theme
                        .font_styles
                        .bigger
                        .rich(format!(
                            "{} {}",
                            localization.get("player"),
                            player_score_info.player_idx.0
                        ))
                        .color(meta.theme.panel.font_color),
                );

                match won {
                    Some(won) => {
                        let text = match won {
                            PlayerWon::Tied => localization.get("tied"),
                            PlayerWon::SoleWinner => localization.get("won"),
                        };

                        ui.label(
                            meta.theme
                                .font_styles
                                .bigger
                                .rich(text)
                                .color(meta.theme.colors.positive),
                        );
                    }
                    None => {
                        ui.add_space(meta.theme.font_styles.bigger.size);
                    }
                }

                let player_meta = assets.get(player_input.selected_player);
                let hat_meta = player_input.selected_hat.map(|x| assets.get(x));
                world.run_system(player_image, (ui, &player_meta, hat_meta.as_deref()));

                ui.label(
                    meta.theme
                        .font_styles
                        .bigger
                        .rich(format!(
                            "{}: {}",
                            localization.get("score"),
                            player_score_info.score,
                        ))
                        .color(meta.theme.panel.font_color),
                );

                if !player_input.is_ai {
                    let (ready_str, color) = match *ready {
                        true => ("ready", meta.theme.colors.positive),
                        false => ("not-ready", meta.theme.colors.negative),
                    };
                    let ready_localized = localization.get(ready_str);
                    ui.label(
                        meta.theme
                            .font_styles
                            .normal
                            .rich(ready_localized)
                            .color(color),
                    );
                } else {
                    ui.label(
                        meta.theme
                            .font_styles
                            .normal
                            .rich(localization.get("ai"))
                            .color(meta.theme.panel.font_color),
                    );
                }
            });
        });
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_scoring_messages(
    network_socket: &(impl NetworkSocket + ?Sized),
    state: &mut ScoringMenuState,
) {
    // TODO handle disconnects
    let datas: Vec<(u32, Vec<u8>)> = network_socket.recv_reliable();
    let local_player_idx = network_socket.player_idx();
    for (_, data) in datas {
        match postcard::from_bytes::<ScoringMessage>(&data) {
            Ok(message) => match message.data {
                ScoringMessageEnum::PlayerReady(player) => {
                    if message.magic == SCORING_MESSAGE_MAGIC && player != local_player_idx {
                        state.ready_players.insert(PlayerIdx(player));
                        debug!("Received message player {} ready", player);
                    }
                }
            },
            Err(e) => warn!("Ignoring network message that was not understood: {e}"),
        }
    }
}
