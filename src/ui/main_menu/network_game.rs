use std::net::{SocketAddr, ToSocketAddrs};

use anyhow::Context;
use async_channel::{Receiver, Sender};
use bevy::tasks::IoTaskPool;
use futures_lite::future;

use crate::networking::NetworkEndpoint;

use super::*;

const MDNS_SERVICE_TYPE: &str = "_jumpy._udp.local.";

static MDNS: Lazy<mdns_sd::ServiceDaemon> = Lazy::new(|| {
    mdns_sd::ServiceDaemon::new().expect("Couldn't start MDNS service discovery thread.")
});

#[derive(SystemParam)]
pub struct MatchmakingMenu<'w, 's> {
    commands: Commands<'w, 's>,
    menu_page: ResMut<'w, MenuPage>,
    game: Res<'w, GameMeta>,
    storage: ResMut<'w, Storage>,
    localization: Res<'w, Localization>,
    state: Local<'s, State>,
    menu_input: Query<'w, 's, &'static mut ActionState<MenuAction>>,
    network_endpoint: Res<'w, NetworkEndpoint>,
}

#[derive(Default)]
pub struct State {
    match_kind: MatchKind,
    lan_service_discovery_recv: Option<mdns_sd::Receiver<mdns_sd::ServiceEvent>>,
    host_info: Option<mdns_sd::ServiceInfo>,
    is_hosting: bool,
    lan_servers: Vec<mdns_sd::ServiceInfo>,
}

#[derive(PartialEq, Eq)]
pub enum MatchKind {
    Lan(LanMode),
    Online,
}

impl Default for MatchKind {
    fn default() -> Self {
        MatchKind::Lan(LanMode::Join)
    }
}

#[derive(Default, PartialEq, Eq)]
pub enum LanMode {
    #[default]
    Join,
    Host {
        service_name: String,
    },
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

        if menu_input.pressed(MenuAction::Back) {
            *params.menu_page = MenuPage::Home;
        }

        // // Transition to player select if match is ready
        // if params.state.status.is_match_ready() {
        //     let status = std::mem::take(&mut params.state.status);

        //     if let Status::MatchReady {
        //         endpoint,
        //         conn,
        //         random_seed,
        //         client_info,
        //     } = status
        //     {
        //         for i in 0..client_info.player_count {
        //             params.player_inputs.players[i].active = true;
        //         }
        //         let client = NetClient::new(endpoint, conn);
        //         params.commands.insert_resource(client);
        //         params.commands.insert_resource(client_info);
        //         *params.menu_page = MenuPage::PlayerSelect;
        //         params.state.status_receiver = default();
        //         params.state.status = default();
        //         params.state.cancel_sender = default();
        //         params.global_rng.reseed(random_seed);
        //     } else {
        //         unreachable!("Programmer error in is_match_ready() helper method");
        //     }
        // }

        let bigger_text_style = &params.game.ui_theme.font_styles.bigger;
        let normal_text_style = &params.game.ui_theme.font_styles.normal;
        let heading_text_style = &params.game.ui_theme.font_styles.heading;
        let normal_button_style = &params.game.ui_theme.button_styles.normal;

        ui.vertical_centered(|ui| {
            ui.add_space(heading_text_style.size / 4.0);
            ui.themed_label(heading_text_style, &params.localization.get("network-game"));
            ui.themed_label(
                bigger_text_style,
                &params.localization.get("configure-match"),
            );
            ui.add_space(heading_text_style.size * 4.0);
        });

        let available_size = ui.available_size();
        let x_margin = available_size.x / 4.0;
        let outer_margin = egui::style::Margin::symmetric(x_margin, 0.0);

        BorderedFrame::new(&params.game.ui_theme.panel.border)
            .margin(outer_margin)
            .padding(params.game.ui_theme.panel.padding.into())
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.horizontal(|ui| {
                    // Lan tab
                    let mut lan = egui::RichText::new(params.localization.get("lan"));
                    if matches!(params.state.match_kind, MatchKind::Lan(..)) {
                        lan = lan.underline();
                    }
                    if BorderedButton::themed(normal_button_style, lan)
                        .show(ui)
                        .clicked()
                    {
                        params.state.match_kind = MatchKind::Lan(default());
                    }

                    // Online tab
                    let mut online = egui::RichText::new(params.localization.get("online"));
                    if params.state.match_kind == MatchKind::Online {
                        online = online.underline();
                    }
                    if BorderedButton::themed(normal_button_style, online)
                        .show(ui)
                        .clicked()
                    {
                        params.state.match_kind = MatchKind::Online
                    }

                    match &mut params.state.match_kind {
                        MatchKind::Lan(mode) => {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.horizontal(|ui| {
                                        // Host tab
                                        let mut host =
                                            egui::RichText::new(params.localization.get("host"));
                                        if matches!(mode, LanMode::Host { .. }) {
                                            host = host.underline();
                                        }
                                        if BorderedButton::themed(
                                            &params.game.ui_theme.button_styles.small,
                                            host,
                                        )
                                        .show(ui)
                                        .clicked()
                                        {
                                            *mode = LanMode::Host {
                                                service_name: params.localization.get("untitled"),
                                            };
                                        }

                                        // Join tab
                                        let mut join =
                                            egui::RichText::new(params.localization.get("join"));
                                        if matches!(mode, LanMode::Join) {
                                            join = join.underline();
                                        }
                                        if BorderedButton::themed(
                                            &params.game.ui_theme.button_styles.small,
                                            join,
                                        )
                                        .show(ui)
                                        .clicked()
                                        {
                                            *mode = LanMode::Join
                                        }
                                    });
                                },
                            );
                        }
                        MatchKind::Online => (),
                    }
                });

                let State {
                    match_kind,
                    lan_service_discovery_recv,
                    lan_servers,
                    host_info,
                    is_hosting,
                } = &mut *params.state;

                ui.separator();
                ui.add_space(normal_text_style.size);

                match match_kind {
                    MatchKind::Lan(mode) => match mode {
                        LanMode::Join => {
                            // Stop any running server
                            if let Some(service_info) = host_info.take() {
                                loop {
                                    match MDNS.unregister(service_info.get_fullname()) {
                                        Ok(_) => break,
                                        Err(mdns_sd::Error::Again) => (),
                                        Err(e) => panic!("Error unregistering MDNS service: {e}"),
                                    }
                                }
                                *is_hosting = false;
                            }

                            let events = lan_service_discovery_recv.get_or_insert_with(|| {
                                MDNS.browse(MDNS_SERVICE_TYPE)
                                    .expect("Couldn't start service discovery")
                            });

                            while let Ok(event) = events.try_recv() {
                                match event {
                                    mdns_sd::ServiceEvent::ServiceResolved(info) => {
                                        lan_servers.push(info)
                                    }
                                    mdns_sd::ServiceEvent::ServiceRemoved(_, full_name) => {
                                        lan_servers.retain(|info| info.get_fullname() != full_name);
                                    }
                                    _ => (),
                                }
                            }

                            ui.themed_label(normal_text_style, &params.localization.get("servers"));
                            ui.add_space(normal_text_style.size / 2.0);

                            ui.indent("servers", |ui| {
                                for server in lan_servers.iter() {
                                    if BorderedButton::themed(
                                        &params.game.ui_theme.button_styles.normal,
                                        server.get_hostname().trim_end_matches('.'),
                                    )
                                    .min_size(egui::vec2(ui.available_width(), 0.0))
                                    .show(ui)
                                    .clicked()
                                    {}
                                }

                                if lan_servers.is_empty() {
                                    ui.themed_label(
                                        normal_text_style,
                                        &params.localization.get("no-servers"),
                                    );
                                }
                            });

                            ui.add_space(normal_text_style.size / 2.0);
                        }
                        LanMode::Host { service_name } => {
                            ui.horizontal(|ui| {
                                ui.label(params.localization.get("server-name"));
                                ui.text_edit_singleline(service_name);
                                *service_name = service_name.replace(' ', "-");
                            });

                            let create_service_info = || {
                                let port = params.network_endpoint.local_addr().unwrap().port();
                                mdns_sd::ServiceInfo::new(
                                    MDNS_SERVICE_TYPE,
                                    service_name,
                                    service_name,
                                    "",
                                    port,
                                    None,
                                )
                                .unwrap()
                                .enable_addr_auto()
                            };

                            let service_info = host_info.get_or_insert_with(create_service_info);

                            if service_info.get_hostname() != service_name {
                                loop {
                                    match MDNS.unregister(service_info.get_fullname()) {
                                        Ok(_) => break,
                                        Err(mdns_sd::Error::Again) => (),
                                        Err(e) => panic!("Error unregistering MDNS service: {e}"),
                                    }
                                }
                                *is_hosting = false;
                                *service_info = create_service_info();
                            }

                            ui.add_space(params.game.ui_theme.font_styles.normal.size);

                            if !*is_hosting {
                                if BorderedButton::themed(
                                    &params.game.ui_theme.button_styles.small,
                                    &params.localization.get("start-server"),
                                )
                                .show(ui)
                                .clicked()
                                {
                                    *is_hosting = true;
                                    MDNS.register(service_info.clone())
                                        .expect("Could not register MDNS service.");
                                }
                            } else if BorderedButton::themed(
                                &params.game.ui_theme.button_styles.small,
                                &params.localization.get("stop-server"),
                            )
                            .show(ui)
                            .clicked()
                            {
                                loop {
                                    match MDNS.unregister(service_info.get_fullname()) {
                                        Ok(_) => break,
                                        Err(mdns_sd::Error::Again) => (),
                                        Err(e) => {
                                            panic!("Error unregistering MDNS service: {e}")
                                        }
                                    }
                                }
                                *is_hosting = false;
                            }
                        }
                    },
                    MatchKind::Online => {}
                }
            });
    }
}
