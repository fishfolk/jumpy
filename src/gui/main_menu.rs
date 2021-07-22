use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{self, *},
};

use crate::gui::GuiResources;

struct Level {
    preview: Texture2D,
    map: String,
    size: f32,
}

pub async fn gui() -> String {
    let mut levels = {
        let gui_resources = storage::get::<GuiResources>();

        vec![
            Level {
                preview: gui_resources.lev01,
                map: "assets/levels/lev01.json".to_string(),
                size: 0.,
            },
            Level {
                preview: gui_resources.lev02,
                map: "assets/levels/lev02.json".to_string(),
                size: 0.,
            },
            Level {
                preview: gui_resources.lev03,
                map: "assets/levels/lev03.json".to_string(),
                size: 0.,
            },
            Level {
                preview: gui_resources.lev04,
                map: "assets/levels/lev04.json".to_string(),
                size: 0.,
            },
            Level {
                preview: gui_resources.lev05,
                map: "assets/levels/lev05.json".to_string(),
                size: 0.,
            },
            Level {
                preview: gui_resources.lev06,
                map: "assets/levels/lev06.json".to_string(),
                size: 0.,
            },
        ]
    };

    let mut hovered: i32 = 0;

    {
        let mut controller = storage::get_mut::<gamepad_rs::ControllerContext>();
        for i in 0..2 {
            controller.update(i);
        }
    }

    {
        let mut input = storage::get_mut::<crate::input_axis::InputAxises>();
        input.update();
    }

    loop {
        let gui_resources = storage::get::<GuiResources>();

        clear_background(BLACK);

        root_ui().push_skin(&gui_resources.skins.main_menu_skin);
        let w = (screen_width() - 120.) / 3. - 50.;
        let h = (screen_height() - 180.) / 2. - 50.;

        {
            let axises = storage::get::<crate::input_axis::InputAxises>();

            if axises.up_pressed {
                hovered -= 3;
            }
            if axises.down_pressed {
                hovered += 3;
                hovered = hovered.max(0);
            }
            if axises.left_pressed {
                hovered -= 1;
            }
            if axises.right_pressed {
                hovered += 1;
            }
            hovered = (hovered + 6) % 6;

            for (n, level) in levels.iter_mut().enumerate() {
                let is_hovered = hovered == n as i32;

                let rect = Rect::new(
                    60. + (n % 3) as f32 * (w + 50.) - level.size * 30.,
                    90. + (n / 3) as f32 * (h + 100.) - level.size * 30.,
                    w + level.size * 60.,
                    h + level.size * 60.,
                );
                if is_hovered {
                    level.size = level.size * 0.8 + 1.0 * 0.2;
                } else {
                    level.size = level.size * 0.9;
                }

                if ui::widgets::Button::new(level.preview)
                    .size(rect.size())
                    .position(rect.point())
                    .ui(&mut *root_ui())
                    || axises.btn_a_pressed
                {
                    root_ui().pop_skin();
                    let level = &levels[hovered as usize];
                    return level.map.clone();
                }
            }
        }

        root_ui().pop_skin();

        {
            let mut controller = storage::get_mut::<gamepad_rs::ControllerContext>();
            for i in 0..2 {
                controller.update(i);
            }
        }

        {
            let mut input = storage::get_mut::<crate::input_axis::InputAxises>();
            input.update();
        }

        next_frame().await;
    }
}
