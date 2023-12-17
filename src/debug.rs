//! Debug tools and menus.

use crate::prelude::*;

pub fn game_plugin(game: &mut Game) {
    game.sessions
        .create(SessionNames::DEBUG)
        .install_plugin(session_plugin);
}

fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::First, debug_menu);
}

#[derive(HasSchema, Clone, Debug, Default)]
struct DebugMenuState {
    pub show_menu: bool,
    pub snapshot: Option<World>,
}

fn debug_menu(
    mut sessions: ResMut<Sessions>,
    keyboard_inputs: Res<KeyboardInputs>,
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
                ui.set_enabled(game_session.is_some());

                if ui.button(localization.get("take-snapshot")).clicked() {
                    if let Some(session) = game_session {
                        *snapshot = Some(session.snapshot());
                    }
                } else if ui.button(localization.get("restore-snapshot")).clicked() {
                    if let Some(session) = game_session {
                        if let Some(mut snapshot) = snapshot.clone() {
                            session.restore(&mut snapshot);
                        }
                    }
                }
            })
        });
}
