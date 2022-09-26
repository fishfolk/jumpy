use leafwing_input_manager::user_input::{InputKind, UserInput};

use crate::input::PlayerAction;

use super::*;

pub fn player_select_ui(params: &mut MenuSystemParams, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        let bigger_text_style = &params.game.ui_theme.font_styles.bigger;
        let heading_text_style = &params.game.ui_theme.font_styles.heading;
        let normal_button_style = &params.game.ui_theme.button_styles.normal;

        ui.add_space(heading_text_style.size / 4.0);
        ui.themed_label(heading_text_style, &params.localization.get("local-game"));
        ui.themed_label(
            bigger_text_style,
            &params.localization.get("player-select-title"),
        );
        ui.add_space(normal_button_style.font.size);

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            let normal_button_style = &params.game.ui_theme.button_styles.normal;

            ui.add_space(normal_button_style.font.size * 2.0);

            ui.horizontal(|ui| {
                // Calculate button size and spacing
                let width = ui.available_width();
                let button_width = width / 3.0;
                let button_min_size = egui::vec2(button_width, 0.0);
                let button_spacing = (width - 2.0 * button_width) / 3.0;

                ui.add_space(button_spacing);

                // Back button
                let back_button = BorderedButton::themed(
                    &params.game.ui_theme.button_styles.normal,
                    &params.localization.get("back"),
                )
                .min_size(button_min_size)
                .show(ui)
                .focus_by_default(ui);

                // Go to menu when back button is clicked
                if back_button.clicked() {
                    *params.menu_page = MenuPage::Main;
                    ui.ctx().clear_focus();
                }

                ui.add_space(button_spacing);

                // Continue button
                let continue_button = BorderedButton::themed(
                    &params.game.ui_theme.button_styles.normal,
                    &params.localization.get("continue"),
                )
                .min_size(button_min_size)
                .show(ui);

                if continue_button.clicked() {
                    *params.menu_page = MenuPage::MapSelect;
                }
            });

            ui.add_space(normal_button_style.font.size);

            ui.vertical_centered(|ui| {
                let normal_button_style = &params.game.ui_theme.button_styles.normal;

                ui.set_width(ui.available_width() - normal_button_style.font.size * 2.0);
                ui.columns(4, |columns| {
                    for (i, ui) in columns.iter_mut().enumerate() {
                        player_select_panel(i, params, ui);
                    }
                });
            });
        });
    });
}

fn player_select_panel(idx: usize, params: &mut MenuSystemParams, ui: &mut egui::Ui) {
    BorderedFrame::new(&params.game.ui_theme.panel.border)
        .padding(params.game.ui_theme.panel.padding.into())
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());

            ui.vertical_centered(|ui| {
                let settings: Option<Settings> = params.storage.get(Settings::STORAGE_KEY);
                let settings = settings.as_ref().unwrap_or(&params.game.default_settings);
                let input_map = settings.player_controls.get_input_map(idx);

                ui.themed_label(
                    &params.game.ui_theme.font_styles.normal,
                    &params.localization.get(&format!("press-to-join")),
                );

                ui.add_space(params.game.ui_theme.button_styles.normal.font.size);

                input_map
                    .get(PlayerAction::Attack)
                    .iter()
                    .map(|x| match x {
                        UserInput::Single(input) => match &input {
                            InputKind::GamepadButton(btn) => {
                                format!(
                                    "{gamepad} {btn:?}",
                                    gamepad = params.localization.get("gamepad")
                                )
                            }
                            InputKind::Keyboard(btn) => format!(
                                "{keyboard} {btn:?}",
                                keyboard = params.localization.get("keyboard")
                            ),
                            i => unimplemented!("Display input kind: {i:?}"),
                        },
                        _ => unimplemented!("Display non-single input type"),
                    })
                    .for_each(|btn| {
                        ui.themed_label(&params.game.ui_theme.font_styles.smaller, &btn);
                    });
            });
        });
}
