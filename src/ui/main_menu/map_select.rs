use super::*;

pub(crate) fn map_select_ui(params: &mut MenuSystemParams, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        let bigger_text_style = &params.game.ui_theme.font_styles.bigger;
        let heading_text_style = &params.game.ui_theme.font_styles.heading;
        let normal_button_style = &params.game.ui_theme.button_styles.normal;

        ui.set_width(ui.available_width() - normal_button_style.font.size * 2.0);

        ui.add_space(heading_text_style.size / 4.0);
        ui.themed_label(heading_text_style, &params.localization.get("local-game"));
        ui.themed_label(
            bigger_text_style,
            &params.localization.get("map-select-title"),
        );
        ui.add_space(normal_button_style.font.size);

        let dummy_map_list = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let tile_size = egui::vec2(240.0, 135.0);
        let cols = ui.available_width().floor() as usize / tile_size.x.ceil() as usize;

        let horizontal_spacing =
            (ui.available_width() - cols as f32 * tile_size.x) / cols as f32 + 1.0;
        let vertical_spacing = normal_button_style.font.size;

        egui::ScrollArea::vertical().show(ui, |ui| {
            for row in 0..(dummy_map_list.len() / cols) {
                ui.add_space(vertical_spacing);

                ui.columns(cols, |columns| {
                    for (col, ui) in columns.iter_mut().enumerate() {
                        ui.set_width(tile_size.x);
                        ui.set_height(tile_size.y);
                        let idx = row + col;

                        if dummy_map_list.get(idx).is_some() {
                            // let padding = &params.game.ui_theme.panel.padding;
                            BorderedFrame::new(&params.game.ui_theme.panel.border)
                                .margin(egui::style::Margin {
                                    left: horizontal_spacing / 2.0,
                                    right: horizontal_spacing / 2.0,
                                    top: 0.0,
                                    bottom: 0.0,
                                })
                                .show(ui, |ui| {
                                    ui.set_height(ui.available_height());
                                    ui.set_width(ui.available_width());
                                });
                        }
                    }
                });
            }

            ui.add_space(vertical_spacing);
        });
    });
}
