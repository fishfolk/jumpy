use bones_framework::render::ui::egui;

use super::*;

pub(super) fn on_cancel(In(state): In<&SettingsState>, mut audio_center: ResMut<AudioCenter>) {
    audio_center.event(AudioEvent::MainVolumeChange(
        state.modified_settings.main_volume,
    ));
}

pub(super) fn widget(
    In((ui, state, should_reset)): In<(&mut egui::Ui, &mut SettingsState, bool)>,
    meta: Root<GameMeta>,
    localization: Localization<GameMeta>,
    mut audio_center: ResMut<AudioCenter>,
) {
    let normal_font = meta
        .theme
        .font_styles
        .normal
        .with_color(meta.theme.panel.font_color);

    if should_reset {
        state.modified_settings.main_volume = meta.default_settings.main_volume;
        audio_center.event(AudioEvent::MainVolumeChange(
            state.modified_settings.main_volume,
        ));
    }

    ui.add_space(normal_font.size);

    ui.scope(|ui| {
        ui.spacing_mut().indent = normal_font.size / 2.5;
        ui.indent("audio-controls", |ui| {
            ui.horizontal(|ui| {
                ui.label(normal_font.rich(localization.get("main-volume")));
                let style = ui.style_mut();
                style.drag_value_text_style = egui::TextStyle::Monospace;
                style.spacing.button_padding = egui::vec2(6.0, 3.0);
                let main_volume = &mut state.modified_settings.main_volume;
                let response = ui.add(
                    egui::Slider::new(main_volume, 0.0..=1.0)
                        .smallest_positive(0.0)
                        .step_by(0.01)
                        .fixed_decimals(3)
                        .custom_formatter(|x, _| format!("{:5.1}", x * 100.0))
                        .custom_parser(|s| s.parse::<f64>().ok().map(|f| f / 100.0)),
                );
                if response.changed() {
                    audio_center.event(AudioEvent::MainVolumeChange(*main_volume));
                }
            });
        });
    });
}
