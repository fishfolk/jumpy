use macroquad::{
    experimental::collections::storage,
    math::{vec2, Vec2},
    ui::{hash, root_ui},
    window::{next_frame, screen_height, screen_width},
};

use super::{GuiResources, Scene, WINDOW_HEIGHT, WINDOW_WIDTH};

pub async fn credits() -> Scene {
    loop {
        let resources = storage::get::<GuiResources>();
        root_ui().push_skin(&resources.login_skin);

        let mut next_scene = None;
        root_ui().window(
            hash!(),
            Vec2::new(
                screen_width() / 2. - WINDOW_WIDTH / 2.,
                screen_height() / 2. - WINDOW_HEIGHT / 2.,
            ),
            Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            |ui| {
                ui.separator();
                ui.label(None, "");
                ui.label(None, "");

                ui.label(None, "Programming: ");
                ui.same_line(220.0);
                ui.label(None, "Fedor Logachev");
                ui.label(None, "Art: ");
                ui.same_line(220.0);
                ui.label(None, "Orlando Herrera a.k.a. Pixel Frog");
                ui.label(None, "Music/Sound: ");
                ui.same_line(220.0);
                ui.label(None, "Jakob T. Rypdal");
                ui.label(None, "Font/UI elements: ");
                ui.same_line(220.0);
                ui.label(None, "Mounir Tohami, mounirtohami.itch.io");

                if ui.button(vec2(560.0, 200.0), "Back") {
                    next_scene = Some(Scene::MainMenu);
                }
            },
        );

        root_ui().pop_skin();

        if let Some(next_scene) = next_scene {
            return next_scene;
        }

        next_frame().await;
    }
}
