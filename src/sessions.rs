use crate::prelude::*;

pub struct SessionNames;
impl SessionNames {
    pub const GAME: &str = "game";
    pub const MAIN_MENU: &str = "main_menu";
    pub const PAUSE_MENU: &str = "pause_menu";
}

pub trait SessionExt {
    fn start_menu(&mut self);
    fn end_game(&mut self);
    fn restart_game(&mut self);
}

impl SessionExt for Sessions {
    fn start_menu(&mut self) {
        self.create(SessionNames::MAIN_MENU)
            .install_plugin(crate::ui::main_menu::session_plugin);
    }

    fn end_game(&mut self) {
        self.delete(SessionNames::GAME);
    }

    #[track_caller]
    fn restart_game(&mut self) {
        if let Some((map, selected_players)) = self.get(SessionNames::GAME).map(|session| {
            let map = (*session.world.resource::<LoadedMap>().0).clone();
            let match_inputs = session.world.resource::<MatchInputs>();
            let selected_players = std::array::from_fn(|i| {
                match_inputs.players[i]
                    .active
                    .then(|| match_inputs.players[i].selected_player)
            });
            (map, selected_players)
        }) {
            self.end_game();
            self.create(SessionNames::GAME)
                .install_plugin(crate::core::MatchPlugin {
                    map,
                    selected_players,
                });
        } else {
            panic!("Cannot restart game when game is not running");
        }
    }
}