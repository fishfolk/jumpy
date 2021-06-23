use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, Handle},
    },
    math::{vec2, Vec2},
    ui::{hash, root_ui, widgets},
    window::{next_frame, screen_height, screen_width},
};

use super::{in_progress_gui, GuiResources, Scene, WINDOW_HEIGHT, WINDOW_WIDTH};

pub async fn authentication(nakama: Handle<Nakama>) -> Scene {
    let mut email = String::new();
    let mut password = String::new();

    let mut email_new = String::new();
    let mut username_new = String::new();
    let mut password_new = String::new();

    let mut authenticating = false;

    loop {
        let resources = storage::get::<GuiResources>();
        root_ui().push_skin(&resources.login_skin);

        let mut nakama = scene::get_node(nakama);

        let mut next_scene = None;
        root_ui().window(
            hash!(),
            Vec2::new(
                screen_width() / 2. - WINDOW_WIDTH / 2.,
                screen_height() / 2. - WINDOW_HEIGHT / 2.,
            ),
            Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            |ui| {
                if nakama.api_client.in_progress() {
                    in_progress_gui(ui, "Authentication", &resources.authenticating_skin);
                    return;
                }
                ui.group(hash!(), vec2(WINDOW_WIDTH / 2. - 28., 170.), |ui| {
                    ui.label(None, "Login");
                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .label("Email")
                        .ui(ui, &mut email);

                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .password(true)
                        .label("Password")
                        .ui(ui, &mut password);

                    ui.separator();

                    if ui.button(None, "Login") {
                        nakama.api_client.authenticate(&email, &password);
                    }
                    // ui.push_skin(&resources.cheat_skin);
                    // if ui.button(None, "Fast cheating login") {
                    //     email = "super@heroes.com".to_owned();
                    //     password = "batsignal".to_owned();
                    // }
                    // ui.pop_skin();
                });
                ui.group(hash!(), vec2(WINDOW_WIDTH / 2. - 28., 170.), |ui| {
                    ui.label(None, "Create an account");
                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .label("Email")
                        .ui(ui, &mut email_new);

                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .label("Username")
                        .ui(ui, &mut username_new);

                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .password(true)
                        .label("Password")
                        .ui(ui, &mut password_new);

                    ui.separator();

                    if ui.button(None, "Register") {
                        authenticating = true;
                        nakama
                            .api_client
                            .register(&email_new, &password_new, &username_new);
                    }
                });

                if let Some(error) = nakama.api_client.error().as_deref() {
                    ui.push_skin(&resources.error_skin);
                    ui.label(None, error);
                    ui.pop_skin();
                }

                if ui.button(vec2(560.0, 200.0), "Back") {
                    next_scene = Some(Scene::MainMenu);
                }
            },
        );

        root_ui().pop_skin();

        if nakama.api_client.authenticated() {
            return Scene::MatchmakingLobby;
        }

        if let Some(next_scene) = next_scene {
            return next_scene;
        }

        drop(nakama);

        next_frame().await;
    }
}
