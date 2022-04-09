use fishsticks::GamepadContext;
use std::path::Path;

use ff_core::prelude::*;

use ff_core::Result;

use ff_core::gui::{get_gui_theme, Panel};

use crate::GuiTheme;
use ff_core::gui::background::draw_main_menu_background;
use ff_core::input::is_gamepad_button_pressed;
use ff_core::macroquad::hash;
use ff_core::macroquad::ui::{root_ui, widgets};
use ff_core::macroquad::window::next_frame;
use ff_core::resources::{
    create_map, map_name_to_filename, MapResource, MAP_EXPORTS_DEFAULT_DIR, MAP_EXPORTS_EXTENSION,
};

enum WindowState {
    None,
    Params(String, String, Vec2, UVec2),
    Cancel,
}

pub async fn show_create_map_menu() -> Result<Option<MapResource>> {
    let mut res = WindowState::None;

    let viewport = get_viewport();

    let size = vec2(350.0, 425.0);

    let position = vec2(
        (viewport.width - size.x) / 2.0,
        (viewport.height - size.y) / 2.0,
    );

    next_frame().await;

    let gui_theme = get_gui_theme();
    root_ui().push_skin(&gui_theme.menu);

    let mut name = "Unnamed Map".to_string();
    let mut description = "".to_string();
    let mut grid_width = "100".to_string();
    let mut grid_height = "100".to_string();
    let mut tile_width = "32".to_string();
    let mut tile_height = "32".to_string();

    let map_export_path = {
        let assets_dir = assets_dir();
        Path::new(&assets_dir).join(MAP_EXPORTS_DEFAULT_DIR)
    };

    loop {
        update_gamepad_context().unwrap();
        draw_main_menu_background(true);

        Panel::new(hash!(), size, position)
            .with_title("Create Map", false)
            .ui(&mut *root_ui(), |ui, _| {
                {
                    let size = vec2(275.0, 25.0);

                    widgets::InputText::new(hash!())
                        .size(size)
                        .ratio(1.0)
                        .ui(ui, &mut name);
                }

                ui.separator();

                {
                    let path_label = map_export_path
                        .join(map_name_to_filename(&name))
                        .with_extension(MAP_EXPORTS_EXTENSION);

                    widgets::Label::new(path_label.to_string_lossy().as_ref()).ui(ui);
                }

                ui.separator();

                {
                    let size = vec2(275.0, 75.0);

                    widgets::InputText::new(hash!())
                        .size(size)
                        .ratio(1.0)
                        .ui(ui, &mut description);
                }

                ui.separator();

                {
                    let size = vec2(75.0, 25.0);

                    widgets::InputText::new(hash!())
                        .size(size)
                        .ratio(1.0)
                        .label("x")
                        .ui(ui, &mut tile_width);

                    ui.same_line(size.x + 25.0);

                    widgets::InputText::new(hash!())
                        .size(size)
                        .ratio(1.0)
                        .label("Tile size")
                        .ui(ui, &mut tile_height);

                    widgets::InputText::new(hash!())
                        .size(size)
                        .ratio(1.0)
                        .label("x")
                        .ui(ui, &mut grid_width);

                    ui.same_line(size.x + 25.0);

                    widgets::InputText::new(hash!())
                        .size(size)
                        .ratio(1.0)
                        .label("Grid size")
                        .ui(ui, &mut grid_height);
                }

                ui.separator();
                ui.separator();
                ui.separator();

                let btn_a = is_gamepad_button_pressed(fishsticks::Button::South);

                let enter = is_key_pressed(KeyCode::Enter);

                if ui.button(None, "Confirm") || btn_a || enter {
                    // TODO: Validate input

                    let tile_size = vec2(
                        tile_width.parse::<f32>().unwrap(),
                        tile_height.parse::<f32>().unwrap(),
                    );

                    let grid_size = uvec2(
                        grid_width.parse::<u32>().unwrap(),
                        grid_height.parse::<u32>().unwrap(),
                    );

                    res = WindowState::Params(
                        name.clone(),
                        description.clone(),
                        tile_size,
                        grid_size,
                    );
                }

                ui.same_line(0.0);

                let btn_b = is_gamepad_button_pressed(fishsticks::Button::East);

                let escape = is_key_pressed(KeyCode::Escape);

                if ui.button(None, "Cancel") || btn_b || escape {
                    res = WindowState::Cancel;
                }
            });

        match res {
            WindowState::Params(name, description, tile_size, grid_size) => {
                root_ui().pop_skin();

                let description = if description.is_empty() {
                    None
                } else {
                    Some(description.as_str())
                };

                return create_map(&name, description, tile_size, grid_size).map(Some);
            }
            WindowState::Cancel => {
                return Ok(None);
            }
            _ => {}
        }

        next_frame().await;
    }
}
