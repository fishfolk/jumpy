use super::*;

pub(super) fn widget(
    mut args: In<(&mut egui::Ui, &mut SettingsState, bool)>,
    meta: Root<GameMeta>,
    localization: Localization<GameMeta>,
) {
    let (ui, state, should_reset) = &mut *args;

    let bigger_font = meta
        .theme
        .font_styles
        .bigger
        .with_color(meta.theme.panel.font_color);
    let normal_font = meta
        .theme
        .font_styles
        .normal
        .with_color(meta.theme.panel.font_color);

    if *should_reset {
        state.modified_settings.matchmaking_server =
            meta.default_settings.matchmaking_server.clone();
    }

    ui.add_space(bigger_font.size / 2.0);

    ui.horizontal(|ui| {
        ui.label(bigger_font.rich(localization.get("matchmaking-server")));

        ui.add(
            egui::TextEdit::singleline(&mut state.modified_settings.matchmaking_server)
                .font(normal_font)
                .desired_width(ui.available_width() - bigger_font.size * 2.0),
        );
    });
}
