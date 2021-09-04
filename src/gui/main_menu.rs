use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{self, hash, root_ui, widgets},
};

use crate::{gui::GuiResources, GameType};

pub async fn game_type() -> GameType {
    let mut self_addr = "127.0.0.1:2323".to_string();
    let mut other_addr = "127.0.0.1:2324".to_string();
    let mut id = "0".to_string();

    let window_width = 700.;
    let window_height = 400.;

    loop {
        let mut res = None;
        let gui_resources = storage::get_mut::<GuiResources>();

        root_ui().push_skin(&gui_resources.skins.login_skin);

        root_ui().window(
            hash!(),
            Vec2::new(
                screen_width() / 2. - window_width / 2.,
                screen_height() / 2. - window_height / 2.,
            ),
            Vec2::new(window_width, window_height),
            |ui| {
                ui.group(hash!(), vec2(window_width / 2. - 28., 170.), |ui| {
                    ui.label(None, "Local game");
                    ui.separator();
                    ui.separator();
                    ui.label(None, "Two players on the same");
                    ui.label(None, "local machine");
                    ui.separator();

                    if ui.button(None, "PLAY") {
                        res = Some(GameType::Local);
                    }
                });
                ui.group(hash!(), vec2(window_width / 2. - 28., 220.), |ui| {
                    ui.label(None, "Network P2P game");
                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .label("Self UDP addr")
                        .ui(ui, &mut self_addr);
                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .label("Remote UDP addr")
                        .ui(ui, &mut other_addr);
                    widgets::InputText::new(hash!())
                        .ratio(3. / 4.)
                        .label("ID (0 or 1)")
                        .ui(ui, &mut id);

                    ui.separator();

                    if ui.button(None, "Connect") {
                        res = Some(GameType::Network {
                            id: id.parse().unwrap(),
                            self_addr: self_addr.clone(),
                            other_addr: other_addr.clone(),
                        });
                    }
                    if ui.button(None, "Connect_dbg") {
                        res = Some(GameType::Network {
                            id: 1,
                            self_addr: "127.0.0.1:2324".to_string(),
                            other_addr: "127.0.0.1:2323".to_string(),
                        });
                    }
                });
            },
        );

        root_ui().pop_skin();

        if let Some(res) = res {
            return res;
        }
        next_frame().await;
    }
}

pub async fn location_select() -> String {
    let mut hovered: i32 = 0;

    let mut old_mouse_position = mouse_position();

    loop {
        let mut gui_resources = storage::get_mut::<GuiResources>();

        clear_background(BLACK);

        let levels_amount = gui_resources.levels.len();

        root_ui().push_skin(&gui_resources.skins.main_menu_skin);

        let rows = (levels_amount + 2) / 3;
        let w = (screen_width() - 120.) / 3. - 50.;
        let h = (screen_height() - 180.) / rows as f32 - 50.;

        {
            if is_key_pressed(KeyCode::Up) {
                hovered -= 3;
                let ceiled_levels_amount = levels_amount as i32 + 3 - (levels_amount % 3) as i32;
                if hovered < 0 {
                    hovered = (hovered + ceiled_levels_amount as i32) % ceiled_levels_amount;
                    if hovered >= levels_amount as i32 {
                        hovered -= 3;
                    }
                }
            }

            if is_key_pressed(KeyCode::Down) {
                hovered += 3;
                if hovered >= levels_amount as i32 {
                    let row = hovered % 3;
                    hovered = row;
                }
            }
            if is_key_pressed(KeyCode::Left) {
                hovered -= 1;
            }
            if is_key_pressed(KeyCode::Right) {
                hovered += 1;
            }
            hovered = (hovered + levels_amount as i32) % levels_amount as i32;

            let levels = &mut gui_resources.levels;

            for (n, level) in levels.iter_mut().enumerate() {
                let is_hovered = hovered == n as i32;

                let rect = Rect::new(
                    60. + (n % 3) as f32 * (w + 50.) - level.size * 30.,
                    90. + 25. + (n / 3) as f32 * (h + 50.) - level.size * 30.,
                    w + level.size * 60.,
                    h + level.size * 60.,
                );
                if old_mouse_position != mouse_position() && rect.contains(mouse_position().into())
                {
                    hovered = n as _;
                }

                if is_hovered {
                    level.size = level.size * 0.8 + 1.0 * 0.2;
                } else {
                    level.size = level.size * 0.9;
                }

                if ui::widgets::Button::new(level.preview)
                    .size(rect.size())
                    .position(rect.point())
                    .ui(&mut *root_ui())
                    || is_key_pressed(KeyCode::Enter)
                {
                    root_ui().pop_skin();
                    let level = &levels[hovered as usize];
                    return level.map.clone();
                }
            }
        }

        root_ui().pop_skin();

        old_mouse_position = mouse_position();

        next_frame().await;
    }
}
