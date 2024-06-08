use std::borrow::BorrowMut;

use crate::prelude::*;

pub fn session_plugin(session: &mut Session) {
    #[cfg(not(target_arch = "wasm32"))]
    session.add_system_to_stage(Update, network_disconnect_notify);
}

pub fn network_disconnect_notify(
    meta: Root<GameMeta>,
    ctx: Res<EguiCtx>,
    mut sessions: ResMut<Sessions>,
    world: &World,
) {
    let sessions = sessions.borrow_mut();
    #[allow(unused_assignments)]
    let mut all_players_disconnected = false;

    #[cfg(not(target_arch = "wasm32"))]
    {
        all_players_disconnected = if let Some(game_session) = sessions.get(SessionNames::GAME) {
            if let Some(disconnected_players) =
                game_session.world.resources.get::<DisconnectedPlayers>()
            {
                // Determine if all remote players have been disconnected
                // (Disconnected players should be player count - 1, can not have local player)
                let player_indices = game_session.world.components.get::<PlayerIdx>().borrow();

                !disconnected_players.disconnected_players.is_empty()
                    && player_indices.bitset().bit_count() - 1
                        == disconnected_players.disconnected_players.len()
            } else {
                false
            }
        } else {
            false
        };
    }

    if all_players_disconnected {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(&ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // Calculate a margin
                    let screen_rect = ui.max_rect();
                    let outer_margin = screen_rect.size() * 0.10;
                    let outer_margin = egui::Margin {
                        left: outer_margin.x,
                        right: outer_margin.x,
                        top: outer_margin.y * 3.0,
                        bottom: outer_margin.y,
                    };

                    BorderedFrame::new(&meta.theme.panel.border)
                        .margin(outer_margin)
                        .padding(meta.theme.panel.padding)
                        .show(ui, |ui| {
                            world.run_system(
                                network_disconnect_notify_ui,
                                (ui, sessions.borrow_mut()),
                            )
                        });
                });
            });
    }
}

fn network_disconnect_notify_ui(
    mut param: In<(&mut egui::Ui, &mut Sessions)>,
    localization: Localization<GameMeta>,
    meta: Root<GameMeta>,
) {
    let (ui, sessions) = &mut *param;
    ui.vertical_centered(|ui| {
        ui.label(
            meta.theme
                .font_styles
                .heading
                .rich(localization.get("disconnected")),
        );

        ui.add_space(meta.theme.font_styles.normal.size);
        ui.label(
            meta.theme
                .font_styles
                .normal
                .rich(localization.get("disconnected-from-all")),
        );
        ui.add_space(meta.theme.font_styles.normal.size);

        if BorderedButton::themed(&meta.theme.buttons.normal, localization.get("exit-match"))
            .show(ui)
            .focus_by_default(ui)
            .clicked()
        {
            sessions.add_command(Box::new(move |sessions: &mut Sessions| {
                sessions.end_game();
                sessions.start_menu();
            }));
        }
    });
}
