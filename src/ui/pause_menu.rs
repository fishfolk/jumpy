use std::ops::Deref;

#[cfg(not(target_arch = "wasm32"))]
use bones_framework::networking::NetworkMatchSocket;

use crate::{core::JumpyDefaultMatchRunner, prelude::*};

use super::scoring::ScoringMenuState;

#[derive(Clone, Debug, Copy, Default)]
enum PauseMenuPage {
    #[default]
    Pause,
    MapSelect,
    Settings,
}

pub fn session_plugin(session: &mut Session) {
    session.add_system_to_stage(Update, pause_menu_system);
}

#[derive(HasSchema, Debug, Clone, Default)]
struct PauseMenu {
    menu_open: bool,
}

fn pause_menu_system(
    meta: Root<GameMeta>,
    mut sessions: ResMut<Sessions>,
    ctx: Res<EguiCtx>,
    controls: Res<GlobalPlayerControls>,
    world: &World,
    assets: Res<AssetServer>,
    mut pause_menu: ResMutInit<PauseMenu>,
) {
    let mut back_to_menu = false;
    let mut restart_game = false;
    let mut close_pause_menu = false;
    let mut close_settings_menu = false;
    let mut select_map = None;
    if let Some(session) = sessions.get_mut(SessionNames::GAME) {
        let pause_pressed = controls.values().any(|x| x.pause_just_pressed);

        #[cfg(not(target_arch = "wasm32"))]
        let is_online = {
            let socket = session.world.get_resource::<NetworkMatchSocket>().clone();
            socket.is_some()
        };
        #[cfg(target_arch = "wasm32")]
        let is_online = false;

        if !session.active && pause_menu.menu_open {
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
                                        (
                                            ui,
                                            session,
                                            &mut restart_game,
                                            &mut back_to_menu,
                                            &mut close_pause_menu,
                                            is_online,
                                        ),
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
                PauseMenuPage::Settings => {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::none())
                        .show(&ctx, |ui| {
                            world.run_system(
                                super::main_menu::settings::widget,
                                (ui, &mut close_settings_menu),
                            );
                        });
                }
            }
        } else if pause_pressed {
            pause_menu.menu_open = true;
            session.active = false;
        }
    }

    if back_to_menu {
        sessions.end_game();
        sessions.start_menu();
        pause_menu.menu_open = false;
    } else if restart_game {
        sessions.restart_game(None, false);
        pause_menu.menu_open = false;
    } else if let Some(maps) = select_map {
        let match_info = sessions
            .get(SessionNames::GAME)
            .unwrap()
            .world
            .resource::<MatchInputs>()
            .deref()
            .clone();
        sessions.end_game();
        sessions.start_game(crate::core::MatchPlugin {
            maps,
            player_info: std::array::from_fn(|i| PlayerInput {
                control: default(),
                editor_input: default(),
                ..match_info.players[i]
            }),
            plugins: meta.get_plugins(&assets),
            session_runner: Box::<JumpyDefaultMatchRunner>::default(),
            score: default(),
        });
    }

    if close_pause_menu {
        pause_menu.menu_open = false;
    }

    if close_settings_menu {
        ctx.set_state(PauseMenuPage::Pause);
    }
}

fn main_pause_menu(
    mut param: In<(
        &mut egui::Ui,
        &mut Session,
        &mut bool,
        &mut bool,
        &mut bool,
        bool,
    )>,
    meta: Root<GameMeta>,
    localization: Localization<GameMeta>,
    controls: Res<GlobalPlayerControls>,
    scoring_menu: Res<ScoringMenuState>,
) {
    let (ui, session, restart_game, back_to_menu, close_pause_menu, is_online) = &mut *param;

    // Unpause the game
    if controls.values().any(|x| x.pause_just_pressed) {
        if !scoring_menu.active {
            // Do not unpause game session if scoring menu open.
            // TODO: Use some kind of pause stack to track what different systems
            // might want session to remain inactive.
            session.active = true;
        }

        **close_pause_menu = true;
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
            **close_pause_menu = true;
        }

        // Local game buttons
        ui.scope(|ui| {
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

            // Settings button
            if BorderedButton::themed(&meta.theme.buttons.normal, localization.get("settings"))
                .min_size(vec2(width, 0.0))
                .show(ui)
                .clicked()
            {
                ui.ctx().set_state(PauseMenuPage::Settings);
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
                **close_pause_menu = true;
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
