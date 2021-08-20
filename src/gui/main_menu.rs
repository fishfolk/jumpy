use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{self, *},
};

use crate::gui::GuiResources;

pub async fn gui() -> String {
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
            let axises = storage::get::<crate::input_axis::InputAxises>();

            if axises.up_pressed {
                hovered -= 3;
                let ceiled_levels_amount = levels_amount as i32 + 3 - (levels_amount % 3) as i32;
                if hovered < 0 {
                    hovered = (hovered + ceiled_levels_amount as i32) % ceiled_levels_amount;
                    if hovered >= levels_amount as i32 {
                        hovered -= 3;
                    }
                }
            }

            if axises.down_pressed {
                hovered += 3;
                if hovered >= levels_amount as i32 {
                    let row = hovered % 3;
                    hovered = row;
                }
            }
            if axises.left_pressed {
                hovered -= 1;
            }
            if axises.right_pressed {
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
                    level.size *= 0.9;
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

        old_mouse_position = mouse_position();

        next_frame().await;
    }
}
