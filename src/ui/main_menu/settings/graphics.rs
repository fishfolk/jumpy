use super::*;

pub(super) fn widget(
    mut args: In<(&mut egui::Ui, &mut SettingsState, bool)>,
    meta: Root<GameMeta>,
    localization: Localization<GameMeta>,
) {
    let (ui, state, should_reset) = &mut *args;

    let normal_font = meta
        .theme
        .font_styles
        .normal
        .with_color(meta.theme.panel.font_color);

    if *should_reset {
        state.modified_settings.fullscreen = meta.default_settings.fullscreen;
    }

    ui.add_space(normal_font.size / 2.0);

    #[cfg(not(target_arch = "wasm32"))]
    ui.horizontal(|ui| {
        ui.add_space(normal_font.size * 3.0);
        ui.checkbox(
            &mut state.modified_settings.fullscreen,
            normal_font.rich(localization.get("fullscreen")),
        );
    });

    #[cfg(target_arch = "wasm32")]
    ui.label(normal_font.rich(localization.get("no-graphics-settings-on-web")));
}
