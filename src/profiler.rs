//! Game plugin for performance profiling tools.

use crate::prelude::*;

/// Installs profiler ui plugins
pub fn game_plugin(game: &mut Game) {
    game.sessions
        .create(SessionNames::PROFILER)
        .install_plugin(session_plugin);
}

/// Designate session that marks profiler's frame boundary.
///
/// Installing to game session is recommended, however this will not work if
/// trying to profile outside of match or while game session is paused.
pub fn install_frame_marker(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::First, mark_new_frame);
}

/// Install the profiler UI to profiler session.
fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::First, profiler);
}

#[derive(HasSchema, Clone, Debug, Default)]
struct ProfilerState {
    pub show_window: bool,
}

fn profiler(
    ctx: ResMut<EguiCtx>,
    localization: Localization<GameMeta>,
    mut state: ResMutInit<ProfilerState>,
    keyboard_inputs: Res<KeyboardInputs>,
) {
    puffin::set_scopes_on(true);

    let toggle_window = keyboard_inputs
        .key_events
        .iter()
        .any(|x| x.key_code.option() == Some(KeyCode::F10) && !x.button_state.pressed());

    let show_window = &mut state.show_window;
    if toggle_window {
        *show_window = !*show_window;
    }

    egui::Window::new(localization.get("profiler"))
        .id(egui::Id::new("profiler"))
        .open(show_window)
        .show(&ctx, |ui| {
            puffin_egui::profiler_ui(ui);
        });
}

/// Notify profilers we are at frame boundary
pub fn mark_new_frame() {
    puffin::GlobalProfiler::lock().new_frame();
}
