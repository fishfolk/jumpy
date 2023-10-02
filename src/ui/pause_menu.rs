use crate::prelude::*;

pub fn session_plugin(session: &mut Session) {
    session.add_system_to_stage(Update, pause_menu_system);
}

fn pause_menu_system(
    meta: Root<GameMeta>,
    mut sessions: ResMut<Sessions>,
    ctx: Egui,
    controls: Res<GlobalPlayerControls>,
    localization: Localization<GameMeta>,
) {
    let mut back_to_menu = false;
    let mut restart_game = false;
    if let Some(session) = sessions.get_mut(SessionNames::GAME) {
        let pause_pressed = controls.iter().any(|x| x.pause_just_pressed);

        if !session.active {
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(&ctx, |ui| {
                    ui.heading("Paused");
                });

            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show(&ctx, |ui| {
                    let screen_rect = ui.max_rect();

                    let pause_menu_width = meta.main_menu.menu_width;
                    let x_margin = (screen_rect.width() - pause_menu_width) / 2.0;
                    let outer_margin =
                        egui::style::Margin::symmetric(x_margin, screen_rect.height() * 0.2);

                    BorderedFrame::new(&meta.theme.panel.border)
                        .margin(outer_margin)
                        .padding(meta.theme.panel.padding.into())
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());

                            ui.vertical_centered(|ui| {
                                let width = ui.available_width();

                                // if let Some(map_meta) = map_handle
                                //     .get_single()
                                //     .ok()
                                //     .and_then(|handle| map_assets.get(handle))
                                // {
                                //     ui.themed_label(&bigger_font, &map_meta.name);
                                // }

                                // Heading
                                ui.label(
                                    meta.theme
                                        .font_styles
                                        .heading
                                        .rich(localization.get("paused"))
                                        .color(meta.theme.panel.font_color),
                                );
                                ui.add_space(10.0);

                                // Continue button
                                let mut continue_button = BorderedButton::themed(
                                    &meta.theme.buttons.normal,
                                    localization.get("continue"),
                                )
                                .min_size(vec2(width, 0.0))
                                .show(ui);
                                continue_button = continue_button.focus_by_default(ui);
                                if continue_button.clicked() {
                                    session.active = true;
                                }

                                // Local game buttons
                                ui.scope(|ui| {
                                    let is_online = false;
                                    ui.set_enabled(!is_online);

                                    // Map select button
                                    // if BorderedButton::themed(
                                    //     &meta.theme.button_styles.normal,
                                    //     &localization.get("map-select-title"),
                                    // )
                                    // .min_size(egui::vec2(width, 0.0))
                                    // .show(ui)
                                    // .clicked()
                                    // {
                                    //     *pause_page = PauseMenuPage::MapSelect;
                                    // }

                                    // Restart button
                                    if BorderedButton::themed(
                                        &meta.theme.buttons.normal,
                                        localization.get("restart"),
                                    )
                                    .min_size(vec2(width, 0.0))
                                    .show(ui)
                                    .clicked()
                                    {
                                        restart_game = true;
                                    }
                                });

                                // Edit button
                                ui.scope(|ui| {
                                    if BorderedButton::themed(
                                        &meta.theme.buttons.normal,
                                        localization.get("edit"),
                                    )
                                    .min_size(vec2(width, 0.0))
                                    .show(ui)
                                    .clicked()
                                    {
                                        // TODO: show editor.
                                        session.active = true;
                                    }
                                });

                                // Main menu button
                                if BorderedButton::themed(
                                    &meta.theme.buttons.normal,
                                    localization.get("main-menu"),
                                )
                                .min_size(vec2(width, 0.0))
                                .show(ui)
                                .clicked()
                                {
                                    back_to_menu = true;
                                }
                            });
                        });
                });

            if pause_pressed {
                session.active = true;
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
    }
}
