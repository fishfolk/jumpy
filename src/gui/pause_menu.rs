use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{hash, root_ui, widgets},
};

use crate::gui::GuiResources;

pub enum PauseResult {
    Quit,
    Close,
    Nothing,
}

pub fn gui() -> PauseResult {
    let gui_resources = storage::get::<GuiResources>();

    let mut res = PauseResult::Nothing;
    root_ui().push_skin(&gui_resources.skins.login_skin);
    widgets::Window::new(
        hash!(),
        vec2(screen_width() / 2. - 100., screen_height() / 2. - 60.),
        vec2(200., 120.),
    )
    .titlebar(false)
    .ui(&mut *root_ui(), |ui| {
        ui.label(None, "Exit, you sure?");
        if ui.button(None, "yes") || is_key_pressed(KeyCode::Enter) {
            res = PauseResult::Quit;
        }
        ui.same_line(90.);
        if ui.button(None, "no") || is_key_pressed(KeyCode::Escape) {
            res = PauseResult::Close;
        }
    });
    root_ui().pop_skin();

    res
}
