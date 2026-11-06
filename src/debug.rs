//! Debug tools and menus.

use crate::prelude::*;
use bones_framework::debug::frame_time_diagnostics_plugin;

#[cfg(not(target_arch = "wasm32"))]
use bones_framework::networking::debug::network_debug_window;

pub fn game_plugin(game: &mut Game) {
    game.sessions
        .create_with(SessionNames::DEBUG, |builder: &mut SessionBuilder| {
            builder
                .install_plugin(session_plugin)
                .install_plugin(frame_time_diagnostics_plugin);
        });
}

fn session_plugin(session: &mut SessionBuilder) {
    session
        .stages
        .add_system_to_stage(CoreStage::First, debug_menu);

    #[cfg(not(target_arch = "wasm32"))]
    session
        .stages
        .add_system_to_stage(CoreStage::First, network_debug_window);
}

#[derive(HasSchema, Clone, Debug, Default)]
struct DebugMenuState {
    pub show_menu: bool,
    pub snapshot: Option<World>,
}

fn debug_menu(
    mut sessions: ResMut<Sessions>,
    keyboard_inputs: Res<KeyboardInputs>,
    mouse_world_pos: Res<MouseWorldPosition>,
    mouse_screen_pos: Res<MouseScreenPosition>,
    mut state: ResMutInit<DebugMenuState>,
    ctx: ResMut<EguiCtx>,
    localization: Localization<GameMeta>,
) {
    let DebugMenuState {
        snapshot,
        show_menu,
    } = &mut *state;

    let toggle_debug = keyboard_inputs
        .key_events
        .iter()
        .any(|x| x.key_code.option() == Some(KeyCode::F12) && !x.button_state.pressed());

    if toggle_debug {
        *show_menu = !*show_menu;
    }
    let game_session = sessions.get_mut(SessionNames::GAME);

    // Delete the snapshot if there is one and we are not in the middle of a game.
    if game_session.is_none() && snapshot.is_some() {
        *snapshot = None;
    }

    egui::Window::new(localization.get("debug-tools"))
        .id(egui::Id::from("debug"))
        .open(show_menu)
        .show(&ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(pos) = mouse_world_pos.0 {
                    ui.label(format!("Mouse World: ({:.1}, {:.1})", pos.x, pos.y));
                } else {
                    ui.label("Mouse World: None");
                }

                if let Some(pos) = mouse_screen_pos.0 {
                    ui.label(format!("Mouse Screen: ({:.1}, {:.1})", pos.x, pos.y));
                } else {
                    ui.label("Mouse Screen: None");
                }
            });

            ui.horizontal(|ui| {
                if game_session.is_none() {
                    ui.disable();
                }

                if ui.button(localization.get("take-snapshot")).clicked() {
                    if let Some(session) = game_session {
                        *snapshot = Some(session.world.clone());
                    }
                } else if ui.button(localization.get("restore-snapshot")).clicked() {
                    if let Some(session) = game_session {
                        if let Some(mut snapshot) = snapshot.clone() {
                            std::mem::swap(&mut session.world, &mut snapshot);
                        }
                    }
                }

                let show_frame_time_window = &mut ctx
                    .get_state::<bones_framework::debug::FrameTimeWindowState>()
                    .open;
                if ui
                    .button(localization.get("frame-time-diagnostics"))
                    .clicked()
                {
                    *show_frame_time_window = !*show_frame_time_window;
                    ctx.set_state(bones_framework::debug::FrameTimeWindowState {
                        open: *show_frame_time_window,
                    });
                }

                // Show net diagnostics button
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let show_network_debug = &mut ctx.get_state::<NetworkDebugMenuState>().open;
                    if ui.button(localization.get("network-debug")).clicked() {
                        *show_network_debug = !*show_network_debug;
                        // Set state so other debug menus may access
                        ctx.set_state(NetworkDebugMenuState {
                            open: *show_network_debug,
                        });
                    }
                }

                // Network debug disabled in wasm.
                #[cfg(target_arch = "wasm32")]
                ui.add_enabled_ui(false, |ui| {
                    let _ = ui.button(localization.get("network-debug"));
                });
            })
        });
}
