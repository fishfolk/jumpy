use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use bevy::utils::Instant;
use smallvec::SmallVec;

use crate::networking::{NetworkMatchSocket, LAN_MATCHMAKER, NETWORK_ENDPOINT};

use super::*;

const MDNS_SERVICE_TYPE: &str = "_jumpy._udp.local.";

static MDNS: Lazy<mdns_sd::ServiceDaemon> = Lazy::new(|| {
    mdns_sd::ServiceDaemon::new().expect("Couldn't start MDNS service discovery thread.")
});

#[derive(DerefMut, Deref)]
pub struct Pinger(
    BiChannelClient<SmallVec<[Ipv4Addr; 10]>, SmallVec<[(Ipv4Addr, Option<u16>); 10]>>,
);

static PINGER: Lazy<Pinger> = Lazy::new(|| {
    let (client, server) = bi_channel();

    std::thread::spawn(move || {
        while let Ok(servers) = server.recv_blocking() {
            let mut pings = SmallVec::new();
            for server in servers {
                let start = Instant::now();
                let ping_result = ping_rs::send_ping(
                    &IpAddr::V4(server),
                    Duration::from_secs(2),
                    &[1, 2, 3, 4],
                    None,
                );

                let ping = if let Err(e) = ping_result {
                    warn!("Error pinging {server}: {e:?}");
                    None
                } else {
                    Some((Instant::now() - start).as_millis() as u16)
                };

                pings.push((server, ping));
            }
            if server.send_blocking(pings).is_err() {
                break;
            }
        }
    });

    Pinger(client)
});

#[derive(SystemParam)]
pub struct MatchmakingMenu<'w, 's> {
    time: Res<'w, Time>,
    menu_page: ResMut<'w, MenuPage>,
    game: Res<'w, GameMeta>,
    localization: Res<'w, Localization>,
    state: Local<'s, State>,
    menu_input: Query<'w, 's, &'static mut ActionState<MenuAction>>,
    commands: Commands<'w, 's>,
}

pub struct State {
    match_kind: MatchKind,
    lan_service_discovery_recv: Option<mdns_sd::Receiver<mdns_sd::ServiceEvent>>,
    service_info: Option<mdns_sd::ServiceInfo>,
    status: Status,
    joined_players: usize,
    lan_servers: Vec<ServerInfo>,
    ping_update_timer: Timer,
}

#[derive(Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Idle,
    Joining,
    Hosting,
}

impl Default for State {
    fn default() -> Self {
        Self {
            match_kind: default(),
            lan_service_discovery_recv: default(),
            service_info: default(),
            status: default(),
            lan_servers: default(),
            joined_players: default(),
            ping_update_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        }
    }
}

pub struct ServerInfo {
    pub service: mdns_sd::ServiceInfo,
    /// The ping in milliseconds
    pub ping: Option<u16>,
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
        player_count: usize,
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
        params.state.ping_update_timer.tick(params.time.delta());

        let bigger_text_style = &params.game.ui_theme.font_styles.bigger;
        let normal_text_style = &params.game.ui_theme.font_styles.normal;
        let heading_text_style = &params.game.ui_theme.font_styles.heading;
        let normal_button_style = &params.game.ui_theme.button_styles.normal;
        let small_button_style = &params.game.ui_theme.button_styles.small;

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
                                                service_name: params.localization.get("fish-fight"),
                                                player_count: 2,
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
                    service_info: host_info,
                    status,
                    ping_update_timer,
                    joined_players,
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
                                *status = Status::Idle;
                            }

                            // Update server pings
                            if ping_update_timer.finished() {
                                PINGER
                                    .try_send(
                                        lan_servers
                                            .iter()
                                            .map(|x| {
                                                *x.service.get_addresses().iter().next().unwrap()
                                            })
                                            .collect(),
                                    )
                                    .ok();
                            }
                            if let Ok(pings) = PINGER.try_recv() {
                                for (server, ping) in pings {
                                    for info in lan_servers.iter_mut() {
                                        if info.service.get_addresses().contains(&server) {
                                            info.ping = ping;
                                        }
                                    }
                                }
                            }

                            let events = lan_service_discovery_recv.get_or_insert_with(|| {
                                MDNS.browse(MDNS_SERVICE_TYPE)
                                    .expect("Couldn't start service discovery")
                            });

                            while let Ok(event) = events.try_recv() {
                                match event {
                                    mdns_sd::ServiceEvent::ServiceResolved(info) => lan_servers
                                        .push(ServerInfo {
                                            service: info,
                                            ping: None,
                                        }),
                                    mdns_sd::ServiceEvent::ServiceRemoved(_, full_name) => {
                                        lan_servers.retain(|server| {
                                            server.service.get_fullname() != full_name
                                        });
                                    }
                                    _ => (),
                                }
                            }

                            if *status != Status::Joining {
                                ui.themed_label(
                                    normal_text_style,
                                    &params.localization.get("servers"),
                                );
                                ui.add_space(normal_text_style.size / 2.0);

                                ui.indent("servers", |ui| {
                                    for server in lan_servers.iter() {
                                        ui.horizontal(|ui| {
                                            if BorderedButton::themed(
                                                &params.game.ui_theme.button_styles.normal,
                                                server.service.get_hostname().trim_end_matches('.'),
                                            )
                                            .min_size(egui::vec2(ui.available_width() * 0.8, 0.0))
                                            .show(ui)
                                            .clicked()
                                            {
                                                *status = Status::Joining;
                                                LAN_MATCHMAKER.try_send(
                                                    networking::LanMatchmakerRequest::JoinServer {
                                                        ip: *server
                                                            .service
                                                            .get_addresses()
                                                            .iter()
                                                            .next()
                                                            .unwrap(),
                                                        port: server.service.get_port(),
                                                    },
                                                ).unwrap();
                                            }

                                            let label_text = egui::RichText::new(format!(
                                                "🖧 {}ms",
                                                server
                                                    .ping
                                                    .map(|x| x.to_string())
                                                    .unwrap_or("?".into())
                                            ))
                                            .size(normal_text_style.size);
                                            ui.label(label_text)
                                        });
                                    }

                                    if lan_servers.is_empty() {
                                        ui.themed_label(
                                            normal_text_style,
                                            &params.localization.get("no-servers"),
                                        );
                                    }
                                });

                            // If we are trying to join a match.
                            } else {
                                ui.themed_label(
                                    normal_text_style,
                                    &params.localization.get("joining"),
                                );

                                while let Ok(message) = LAN_MATCHMAKER.try_recv() {
                                    match message {
                                        networking::LanMatchmakerResponse::ServerStarted => (),
                                        networking::LanMatchmakerResponse::PlayerCount(_) => (),
                                        networking::LanMatchmakerResponse::GameStarting {
                                            lan_socket,
                                            player_idx,
                                            player_count: _,
                                        } => {
                                            info!(?player_idx, "Starting network game");
                                            params.commands.insert_resource(NetworkMatchSocket(
                                                Box::new(lan_socket),
                                            ));

                                            *status = default();
                                            *params.menu_page = MenuPage::PlayerSelect;
                                        }
                                    }
                                }
                            }

                            ui.add_space(normal_text_style.size / 2.0);
                        }
                        LanMode::Host {
                            service_name,
                            player_count,
                        } => {
                            ui.scope(|ui| {
                                ui.set_enabled(*status != Status::Hosting);
                                ui.horizontal(|ui| {
                                    ui.themed_label(
                                        normal_text_style,
                                        &params.localization.get("server-name"),
                                    );
                                    ui.add(
                                        egui::TextEdit::singleline(service_name)
                                            .font(normal_text_style.font_id()),
                                    );
                                    *service_name = service_name.replace(' ', "-");
                                });
                                ui.add_space(normal_text_style.size / 2.0);
                                ui.horizontal(|ui| {
                                    ui.themed_label(
                                        normal_text_style,
                                        &params.localization.get("player-count"),
                                    );
                                    ui.add_space(normal_text_style.size);
                                    ui.scope(|ui| {
                                        ui.set_enabled(*player_count > 2);
                                        if BorderedButton::themed(small_button_style, "-")
                                            .min_size(egui::vec2(normal_text_style.size * 2.0, 0.0))
                                            .show(ui)
                                            .clicked()
                                        {
                                            *player_count = player_count
                                                .saturating_sub(1)
                                                .clamp(2, MAX_PLAYERS);
                                        }
                                    });
                                    ui.themed_label(normal_text_style, &player_count.to_string());
                                    ui.scope(|ui| {
                                        ui.set_enabled(*player_count < MAX_PLAYERS);
                                        if BorderedButton::themed(small_button_style, "+")
                                            .min_size(egui::vec2(normal_text_style.size * 2.0, 0.0))
                                            .show(ui)
                                            .clicked()
                                        {
                                            *player_count = player_count
                                                .saturating_add(1)
                                                .clamp(2, MAX_PLAYERS);
                                        }
                                    });

                                    *service_name = service_name.replace(' ', "-");
                                });
                            });

                            let create_service_info = || {
                                let port = NETWORK_ENDPOINT.local_addr().unwrap().port();
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
                                *status = Status::Idle;
                                *service_info = create_service_info();
                            }

                            ui.add_space(params.game.ui_theme.font_styles.normal.size);

                            if *status == Status::Idle {
                                if BorderedButton::themed(
                                    small_button_style,
                                    &params.localization.get("start-server"),
                                )
                                .show(ui)
                                .clicked()
                                {
                                    *status = Status::Hosting;
                                    MDNS.register(service_info.clone())
                                        .expect("Could not register MDNS service.");
                                    LAN_MATCHMAKER
                                        .try_send(networking::LanMatchmakerRequest::StartServer {
                                            player_count: *player_count,
                                        })
                                        .unwrap();
                                }

                            // If we are hosting a match currently
                            } else if *status == Status::Hosting {
                                while let Ok(response) = LAN_MATCHMAKER.try_recv() {
                                    match response {
                                        networking::LanMatchmakerResponse::PlayerCount(count) => {
                                            *joined_players = count;
                                        }
                                        networking::LanMatchmakerResponse::GameStarting {
                                            lan_socket,
                                            player_idx,
                                            player_count: _,
                                        } => {
                                            info!(?player_idx, "Starting network game");
                                            params.commands.insert_resource(NetworkMatchSocket(
                                                Box::new(lan_socket),
                                            ));

                                            *status = default();
                                            *params.menu_page = MenuPage::PlayerSelect;
                                            loop {
                                                match MDNS.unregister(service_info.get_fullname()) {
                                                    Ok(_) => break,
                                                    Err(mdns_sd::Error::Again) => (),
                                                    Err(e) => panic!(
                                                        "Error unregistering MDNS service: {e}"
                                                    ),
                                                }
                                            }
                                        }
                                        _ => (),
                                    }
                                }

                                ui.horizontal(|ui| {
                                    if BorderedButton::themed(
                                        small_button_style,
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
                                        *status = Status::Idle;
                                    }

                                    ui.themed_label(
                                        normal_text_style,
                                        &format!(
                                            "{} {} / {}",
                                            &params.localization.get("players"),
                                            *joined_players + 1, // Add one to count the host.
                                            player_count
                                        ),
                                    );
                                });
                            }
                        }
                    },
                    MatchKind::Online => {}
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if BorderedButton::themed(normal_button_style, &params.localization.get("back"))
                        .show(ui)
                        .clicked()
                        || menu_input.pressed(MenuAction::Back)
                    {
                        match status {
                            Status::Idle => (),
                            Status::Joining => {
                                LAN_MATCHMAKER
                                    .try_send(networking::LanMatchmakerRequest::StopJoin)
                                    .unwrap();
                                *status = Status::Idle;
                            }
                            Status::Hosting => {
                                if let Some(service_info) = host_info.take() {
                                    loop {
                                        match MDNS.unregister(service_info.get_fullname()) {
                                            Ok(_) => break,
                                            Err(mdns_sd::Error::Again) => (),
                                            Err(e) => {
                                                panic!("Error unregistering MDNS service: {e}")
                                            }
                                        }
                                    }
                                }
                                *status = Status::Idle;
                            }
                        }
                        *params.menu_page = MenuPage::Home;
                    }
                });
            });
    }
}
