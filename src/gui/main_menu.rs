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

    let mut hovered: Option<usize> = None;

    loop {
        let gui_resources = storage::get::<GuiResources>();

        clear_background(BLACK);

        root_ui().push_skin(&gui_resources.skins.main_menu_skin);
        let w = (screen_width() - 120.) / 3. - 50.;
        let h = (screen_height() - 180.) / 2. - 50.;

        for (n, level) in levels.iter_mut().enumerate() {
            let is_hovered = hovered.map_or(false, |h| h == n);

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

            if rect.contains(vec2(mouse_position().0, mouse_position().1)) {
                hovered = Some(n);
            }

            if ui::widgets::Button::new(level.preview)
                .size(rect.size())
                .position(rect.point())
                .ui(&mut *root_ui())
            {
                root_ui().pop_skin();
                return level.map.clone();
            }
        }

        root_ui().pop_skin();

        next_frame().await;
    }
}
