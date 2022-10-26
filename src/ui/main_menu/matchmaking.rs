use async_channel::{Receiver, Sender};
use bevy::tasks::IoTaskPool;
use futures_lite::future;
use jumpy_matchmaker_proto::{MatchInfo, MatchmakerRequest, MatchmakerResponse};
use quinn::{Connection, Endpoint};

use crate::{config::ENGINE_CONFIG, networking::client::NetClient, player::MAX_PLAYERS};

use super::*;

#[derive(SystemParam)]
pub struct MatchmakingMenu<'w, 's> {
    commands: Commands<'w, 's>,
    menu_page: ResMut<'w, MenuPage>,
    game: Res<'w, GameMeta>,
    localization: Res<'w, Localization>,
    state: Local<'s, State>,
    menu_input: Query<'w, 's, &'static mut ActionState<MenuAction>>,
}

pub struct State {
    player_count: u8,
    status: Status,
    status_receiver: Option<Receiver<Status>>,
    cancel_sender: Option<Sender<()>>,
}

#[derive(Debug, Clone, Default)]
enum Status {
    #[default]
    NotConnected,
    Connecting,
    Connected,
    WaitingForPlayers {
        players: usize,
    },
    MatchReady(Endpoint, Connection),
    Errored(String),
}

impl Status {
    fn is_not_connected(&self) -> bool {
        matches!(self, Status::NotConnected) || matches!(self, Status::Errored(_))
    }
    fn is_match_ready(&self) -> bool {
        matches!(self, Status::MatchReady(_, _))
    }
}

impl State {
    fn update_status(&mut self) {
        if let Some(receiver) = &mut self.status_receiver {
            while let Ok(status) = receiver.try_recv() {
                self.status = status;
            }
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            player_count: 4,
            status: Status::NotConnected,
            status_receiver: None,
            cancel_sender: None,
        }
    }
}

impl<'w, 's> WidgetSystem for MatchmakingMenu<'w, 's> {
    type Args = ();
    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut egui::Ui,
        _: WidgetId,
        _: (),
    ) {
        let mut params: MatchmakingMenu = state.get_mut(world);
        let menu_input = params.menu_input.single();
        params.state.update_status();

        // Transition to player select if match is ready
        if params.state.status.is_match_ready() {
            let status = std::mem::take(&mut params.state.status);

            if let Status::MatchReady(endpoint, conn) = status {
                let client = NetClient::new(endpoint, conn);
                params.commands.insert_resource(client);
                *params.menu_page = MenuPage::PlayerSelect;
                params.state.status_receiver = default();
                params.state.status = default();
                params.state.cancel_sender = default();
            } else {
                unreachable!("Programmer error in is_match_ready() helper method");
            }
        }

        let bigger_text_style = &params.game.ui_theme.font_styles.bigger;
        let normal_text_style = &params.game.ui_theme.font_styles.normal;
        let heading_text_style = &params.game.ui_theme.font_styles.heading;
        let normal_button_style = &params.game.ui_theme.button_styles.normal;

        ui.vertical_centered(|ui| {
            ui.add_space(heading_text_style.size / 4.0);
            ui.themed_label(heading_text_style, &params.localization.get("online-game"));
            ui.themed_label(
                bigger_text_style,
                &params.localization.get("configure-match"),
            );
            ui.add_space(heading_text_style.size * 4.0);
        });

        let available_size = ui.available_size();
        let menu_width = params.game.main_menu.menu_width;
        let x_margin = (available_size.x - menu_width) / 2.0;
        let outer_margin = egui::style::Margin::symmetric(x_margin, 0.0);

        BorderedFrame::new(&params.game.ui_theme.panel.border)
            .margin(outer_margin)
            .padding(params.game.ui_theme.panel.padding.into())
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.add_space(normal_button_style.font.size);
                ui.horizontal(|ui| {
                    ui.themed_label(
                        bigger_text_style,
                        &format!("{}: ", params.localization.get("player-count")),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.scope(|ui| {
                            ui.set_enabled(
                                params.state.player_count < MAX_PLAYERS as u8
                                    && params.state.status.is_not_connected(),
                            );
                            if BorderedButton::themed(normal_button_style, "+")
                                .focus_on_hover(false)
                                .show(ui)
                                .clicked()
                            {
                                params.state.player_count += 1;
                            }
                        });
                        ui.themed_label(bigger_text_style, &params.state.player_count.to_string());
                        ui.scope(|ui| {
                            ui.set_enabled(
                                params.state.player_count > 1
                                    && params.state.status.is_not_connected(),
                            );
                            if BorderedButton::themed(normal_button_style, "-")
                                .focus_on_hover(false)
                                .show(ui)
                                .clicked()
                            {
                                params.state.player_count -= 1;
                            }
                        });
                    });
                });

                ui.add_space(normal_button_style.font.size);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.scope(|ui| {
                        ui.set_enabled(params.state.status.is_not_connected());
                        if BorderedButton::themed(
                            normal_button_style,
                            &params.localization.get("search-for-match"),
                        )
                        .focus_on_hover(false)
                        .show(ui)
                        .clicked()
                        {
                            let io_pool = IoTaskPool::get();
                            let (status_sender, status_receiver) = async_channel::unbounded();
                            let (cancel_sender, cancel_receiver) = async_channel::bounded(1);

                            params.state.status_receiver = Some(status_receiver);
                            params.state.cancel_sender = Some(cancel_sender);
                            params.state.status = Status::Connecting;
                            io_pool
                                .spawn(start_matchmaking(
                                    status_sender,
                                    cancel_receiver,
                                    params.state.player_count,
                                ))
                                .detach();
                        }
                    });

                    if BorderedButton::themed(
                        normal_button_style,
                        &params.localization.get("cancel"),
                    )
                    .focus_on_hover(false)
                    .show(ui)
                    .clicked()
                        || menu_input.just_pressed(MenuAction::Back)
                    {
                        if let Some(sender) = &mut params.state.cancel_sender {
                            sender.try_send(()).ok();
                        }
                        params.state.status_receiver = default();
                        params.state.status = default();
                        params.state.cancel_sender = default();
                        *params.menu_page = MenuPage::Home;
                    }
                });

                ui.add_space(normal_button_style.font.size * 2.0);

                ui.vertical_centered(|ui| match &params.state.status {
                    Status::NotConnected => (),
                    Status::Connecting => {
                        ui.themed_label(normal_text_style, &params.localization.get("connecting"));
                    }
                    Status::Connected => {
                        ui.themed_label(normal_text_style, &params.localization.get("connected"));
                    }
                    Status::WaitingForPlayers { players } => {
                        ui.themed_label(
                            normal_text_style,
                            &params.localization.get(&format!(
                                "waiting-for-players?current={}&total={}",
                                players, params.state.player_count
                            )),
                        );
                    }
                    Status::MatchReady(_, _) => {
                        // We shouldn't get here because we check for a ready match above
                    }
                    Status::Errored(e) => {
                        ui.themed_label(
                            normal_text_style,
                            &format!("{}: {e}", params.localization.get("error")),
                        );
                    }
                })
            });
    }
}

async fn start_matchmaking(
    sender: Sender<Status>,
    cancel_receiver: Receiver<()>,
    player_count: u8,
) {
    if let Err(e) = impl_start_matchmaking(sender.clone(), cancel_receiver, player_count).await {
        sender.try_send(Status::Errored(e.to_string())).ok();
        error!("Error while matchmaking: {e}");
    }
}

async fn impl_start_matchmaking(
    status: Sender<Status>,
    cancel_receiver: Receiver<()>,
    player_count: u8,
) -> anyhow::Result<()> {
    let server_addr = ENGINE_CONFIG.matchmaking_server;
    let (endpoint, conn) = crate::networking::client::open_connection(server_addr).await?;

    let matchmake = async {
        status.try_send(Status::Connected).ok();

        let (mut send, recv) = conn.open_bi().await?;

        let message = MatchmakerRequest::RequestMatch(MatchInfo { player_count });

        let message = postcard::to_allocvec(&message)?;
        send.write_all(&message).await?;
        send.finish().await?;

        let message = recv.read_to_end(256).await?;
        let message: MatchmakerResponse = postcard::from_bytes(&message)?;

        if let MatchmakerResponse::Accepted = message {
            status
                .try_send(Status::WaitingForPlayers { players: 1 })
                .ok();
        } else {
            status
                .try_send(Status::Errored("Unexpected response from server".into()))
                .ok();
            return Ok::<(), anyhow::Error>(());
        }

        loop {
            let recv = conn.accept_uni().await?;
            let message = recv.read_to_end(256).await?;
            let message: MatchmakerResponse = postcard::from_bytes(&message)?;

            match message {
                MatchmakerResponse::PlayerCount(count) => {
                    status
                        .try_send(Status::WaitingForPlayers {
                            players: count as usize,
                        })
                        .ok();
                }
                MatchmakerResponse::Success => {
                    status
                        .try_send(Status::MatchReady(endpoint, conn.clone()))
                        .ok();
                    break;
                }
                _ => panic!("Unexpected message from server"),
            }
        }

        Ok(())
    };

    match future::or(
        async move { either::Left(cancel_receiver.recv().await) },
        async move { either::Right(matchmake.await) },
    )
    .await
    {
        either::Either::Left(_) => {
            conn.close(0u8.into(), b"canceled");
            Ok(())
        }
        either::Either::Right(result) => result,
    }
}
