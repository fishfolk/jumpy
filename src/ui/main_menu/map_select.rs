use crate::core::MatchPlugin;
use crate::prelude::*;

use crate::ui::map_select::{map_select_menu, MapSelectAction};

use super::player_select::PlayerSelectState;
use super::MenuPage;

pub fn widget(
    ui: In<&mut egui::Ui>,
    world: &World,
    meta: Root<GameMeta>,
    mut sessions: ResMut<Sessions>,
    mut session_options: ResMut<SessionOptions>,
    assets: Res<AssetServer>,
) {
    let select_action = world.run_initialized_system(map_select_menu, ());

    match select_action {
        MapSelectAction::None => (),
        MapSelectAction::SelectMap(map_meta) => {
            session_options.delete = true;
            ui.ctx().set_state(MenuPage::Home);

            let player_select_state = ui.ctx().get_state::<PlayerSelectState>();
            sessions.start_game(MatchPlugin {
                map: map_meta,
                player_info: std::array::from_fn(|i| {
                    let slot = player_select_state.slots[i];

                    PlayerInput {
                        active: slot.active,
                        selected_player: slot.selected_player,
                        selected_hat: slot.selected_hat,
                        control_source: slot.control_source,
                        editor_input: default(),
                        control: default(),
                    }
                }),
                plugins: meta.get_plugins(&assets),
            });
            ui.ctx().set_state(PlayerSelectState::default());
        }
        MapSelectAction::GoBack => ui.ctx().set_state(MenuPage::PlayerSelect),
    }
}
