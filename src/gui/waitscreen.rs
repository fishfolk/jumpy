use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, Handle},
    },
    math::Vec2,
    ui::{hash, root_ui},
    window::{next_frame, screen_height, screen_width},
};

use super::{in_progress_gui, GuiResources, Scene, WINDOW_HEIGHT, WINDOW_WIDTH};

pub async fn waitscreen(nakama: Handle<Nakama>, private: bool) -> Scene {
    let resources = storage::get::<GuiResources>();

    enum State {
        WaitingForMatchmaking,
        WaitingForMatchJoin,
    }
    let mut state = if private {
        State::WaitingForMatchJoin
    } else {
        State::WaitingForMatchmaking
    };

    loop {
        root_ui().push_skin(&resources.login_skin);

        let mut next_scene = None;

        let mut nakama = scene::get_node(nakama);

        root_ui().window(
            hash!(),
            Vec2::new(
                screen_width() / 2. - WINDOW_WIDTH / 2.,
                screen_height() / 2. - WINDOW_HEIGHT / 2.,
            ),
            Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            |ui| match state {
                State::WaitingForMatchmaking => {
                    let token = nakama.api_client.matchmaker_token.clone();
                    if token.is_none() {
                        in_progress_gui(ui, "Looking for a match", &resources.authenticating_skin);
                        return;
                    }
                    if let Some(_error) = nakama.api_client.error().clone() {
                        ui.label(None, "Invalid match ID");
                        if ui.button(None, "Back to matchmaking") {
                            next_scene = Some(Scene::MatchmakingLobby);
                        }
                    } else {
                        nakama
                            .api_client
                            .socket_join_match_by_token(&token.unwrap());
                        state = State::WaitingForMatchJoin;
                    }
                }
                State::WaitingForMatchJoin => {
                    if nakama.api_client.match_id().is_some() {
                        next_scene = Some(Scene::MatchmakingGame { private });
                    }
                }
            },
        );

        root_ui().pop_skin();

        drop(nakama);

        if let Some(scene) = next_scene {
            return scene;
        }

        next_frame().await;
    }
}
