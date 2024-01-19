use crate::ui::network_game::network_game_menu;

use super::*;

// TODO remove ui param?
pub fn widget(_ui: In<&mut egui::Ui>, world: &World) {
    // let mut params: MatchmakingMenu = state.get_mut(world);
    // let menu_input = params.menu_input.single();

    world.run_system(network_game_menu, ());

    // TODO
    // params.state.ping_update_timer.tick(params.time.delta());
}
