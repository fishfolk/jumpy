use super::*;

pub fn networking_settings_ui(
    params: &mut SettingsMenu,
    ui: &mut egui::Ui,
    should_reset: bool,
    settings_tabs: &[egui::Response],
    bottom_buttons: &[egui::Response],
) {
    let settings = params.modified_settings.0.as_mut().unwrap();

    let bigger_font = &params.game.ui_theme.font_styles.bigger;
    let normal_font = &params.game.ui_theme.font_styles.normal;

    ui.add_space(bigger_font.size);

    ui.horizontal(|ui| {
        ui.add_space(bigger_font.size * 2.0);
        ui.themed_label(
            bigger_font,
            &format!("{}:", params.localization.get("matchmaking-server")),
        );

        if should_reset {
            settings.matchmaking_server = params.game.default_settings.matchmaking_server.clone();
        }

        let text_box = &ui.add(
            egui::TextEdit::singleline(&mut settings.matchmaking_server).font(normal_font.clone()),
        );
        let first_bottom_button = bottom_buttons.iter().next().unwrap();
        let last_bottom_button = bottom_buttons.iter().last().unwrap();
        let first_top_tab = settings_tabs.iter().next().unwrap();
        let last_top_tab = settings_tabs.iter().last().unwrap();
        params.adjacencies.text_boxes.insert(text_box.id);

        params.adjacencies.widget(text_box).to_right_of(last_top_tab);
        for tab in settings_tabs {
            params.adjacencies.widget(text_box).below(tab);
            params.adjacencies.widget(tab).below(first_bottom_button);
        }
        for button in bottom_buttons {
            params.adjacencies.widget(button).below(text_box);
        }
        params
            .adjacencies
            .widget(text_box)
            .above(first_bottom_button);
        params
            .adjacencies
            .widget(last_bottom_button)
            .to_left_of(first_top_tab);
    });
}
