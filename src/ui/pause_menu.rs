use std::ops::Deref;

#[cfg(not(target_arch = "wasm32"))]
use bones_framework::networking::NetworkMatchSocket;

use crate::{core::JumpyDefaultMatchRunner, prelude::*};

#[derive(Clone, Debug, Copy, Default)]
enum PauseMenuPage {
    #[default]
    Pause,
    MapSelect,
}

pub fn session_plugin(session: &mut Session) {
    session.add_system_to_stage(Update, pause_menu_system);
}

fn pause_menu_system(
    meta: Root<GameMeta>,
    mut sessions: ResMut<Sessions>,
    ctx: Res<EguiCtx>,
    controls: Res<GlobalPlayerControls>,
    world: &World,
    assets: Res<AssetServer>,

    #[cfg(not(target_arch = "wasm32"))] socket: Option<Res<NetworkMatchSocket>>,
) {
    // TODO allow pause menu in online game
    #[cfg(not(target_arch = "wasm32"))]
    if socket.is_some() {
        return;
    }

    let mut back_to_menu = false;
    let mut restart_game = false;
    let mut select_map = None;
    if let Some(session) = sessions.get_mut(SessionNames::GAME) {
        let pause_pressed = controls.values().any(|x| x.pause_just_pressed);

        if !session.active {
            let page = ctx.get_state::<PauseMenuPage>();

            match page {
                PauseMenuPage::Pause => {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::none())
                        .show(&ctx, |ui| {
                            let screen_rect = ui.max_rect();

                            let pause_menu_width = meta.main_menu.menu_width;
                            let x_margin = (screen_rect.width() - pause_menu_width) / 2.0;
                            let outer_margin = egui::style::Margin::symmetric(
                                x_margin,
                                screen_rect.height() * 0.2,
                            );

                            BorderedFrame::new(&meta.theme.panel.border)
                                .margin(outer_margin)
                                .padding(meta.theme.panel.padding)
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());

                                    world.run_system(
                                        main_pause_menu,
                                        (ui, session, &mut restart_game, &mut back_to_menu),
                                    );
                                });
                        });
                }
                PauseMenuPage::MapSelect => {
                    let action = world.run_system(crate::ui::map_select::map_select_menu, ());

                    match action {
                        super::map_select::MapSelectAction::None => (),
                        super::map_select::MapSelectAction::SelectMap(map) => {
                            select_map = Some(map);
                            ctx.set_state(PauseMenuPage::Pause);
                        }
                        super::map_select::MapSelectAction::GoBack => {
                            ctx.set_state(PauseMenuPage::Pause);
                        }
                    }
                }
            }
        } else if pause_pressed {
            session.active = false;
        }
    }

    if back_to_menu {
        sessions.end_game();
        sessions.start_menu();
    } else if restart_game {
        sessions.restart_game();
    } else if let Some(map_handle) = select_map {
        let map = assets.get(map_handle).clone();

        let match_info = sessions
            .get(SessionNames::GAME)
            .unwrap()
            .world
            .resource::<MatchInputs>()
            .deref()
            .clone();
        sessions.end_game();
        sessions.start_game(crate::core::MatchPlugin {
            map,
            player_info: std::array::from_fn(|i| PlayerInput {
                control: default(),
                editor_input: default(),
                ..match_info.players[i]
            }),
            plugins: meta.get_plugins(&assets),
            session_runner: Box::<JumpyDefaultMatchRunner>::default(),
        })
    }
}

fn main_pause_menu(
    mut param: In<(&mut egui::Ui, &mut Session, &mut bool, &mut bool)>,
    meta: Root<GameMeta>,
    localization: Localization<GameMeta>,
    controls: Res<GlobalPlayerControls>,

    #[cfg(not(target_arch = "wasm32"))] socket: Option<Res<NetworkMatchSocket>>,
) {
    let (ui, session, restart_game, back_to_menu) = &mut *param;

    #[cfg(not(target_arch = "wasm32"))]
    let is_online = socket.is_some();
    #[cfg(target_arch = "wasm32")]
    let is_online = false;

    // Unpause the game
    if controls.values().any(|x| x.pause_just_pressed) {
        session.active = true;
    }

    ui.vertical_centered(|ui| {
        let width = ui.available_width();

        // Heading
        ui.label(
            meta.theme
                .font_styles
                .heading
                .rich(localization.get("paused"))
                .color(meta.theme.panel.font_color),
        );

        // Map title
        if let Some(map_meta) = session.world.get_resource::<SpawnedMapMeta>() {
            ui.label(
                meta.theme
                    .font_styles
                    .bigger
                    .rich(map_meta.name.to_string()),
            );
        }

        ui.add_space(10.0);

        // Continue button
        if BorderedButton::themed(&meta.theme.buttons.normal, localization.get("continue"))
            .min_size(vec2(width, 0.0))
            .show(ui)
            .focus_by_default(ui)
            .clicked()
        {
            session.active = true;
        }

        // Local game buttons
        ui.scope(|ui| {
            ui.set_enabled(!is_online);

            // Map select button
            if BorderedButton::themed(
                &meta.theme.buttons.normal,
                localization.get("map-select-title"),
            )
            .min_size(vec2(width, 0.0))
            .show(ui)
            .clicked()
            {
                ui.ctx().set_state(PauseMenuPage::MapSelect);
            }

            // Restart button
            if BorderedButton::themed(&meta.theme.buttons.normal, localization.get("restart"))
                .min_size(vec2(width, 0.0))
                .show(ui)
                .clicked()
            {
                **restart_game = true;
            }
        });

        // Edit button
        ui.scope(|ui| {
            if BorderedButton::themed(&meta.theme.buttons.normal, localization.get("edit"))
                .min_size(vec2(width, 0.0))
                .show(ui)
                .clicked()
            {
                // TODO: show editor.
                session.active = true;
            }
        });

        // Main menu button
        if BorderedButton::themed(&meta.theme.buttons.normal, localization.get("main-menu"))
            .min_size(vec2(width, 0.0))
            .show(ui)
            .clicked()
        {
            **back_to_menu = true;
        }
    });
}
