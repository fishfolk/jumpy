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

        let text_box_resp = ui.add(
            egui::TextEdit::singleline(&mut settings.matchmaking_server).font(normal_font.clone()),
        );
        params.adjacencies.text_boxes.insert(text_box_resp.id);
        params
            .adjacencies
            .widget(&text_box_resp)
            .below(settings_tabs.iter().last().unwrap());
        params
            .adjacencies
            .widget(&text_box_resp)
            .below(bottom_buttons.iter().next().unwrap());
    });
}
