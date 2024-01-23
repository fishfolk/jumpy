use bones_framework::render::ui::egui;

use super::*;

trait SettingsExts {
    fn volume_change_event(&self) -> AudioEvent;
}

impl SettingsExts for Settings {
    fn volume_change_event(&self) -> AudioEvent {
        AudioEvent::VolumeChange {
            main_volume: self.main_volume,
            music_volume: self.music_volume,
            effects_volume: self.effects_volume,
        }
    }
}

pub(super) fn on_cancel(In(state): In<&SettingsState>, mut audio_center: ResMut<AudioCenter>) {
    audio_center.event(state.modified_settings.volume_change_event());
}

pub(super) fn widget(
    In((ui, state, should_reset)): In<(&mut egui::Ui, &mut SettingsState, bool)>,
    meta: Root<GameMeta>,
    localization: Localization<GameMeta>,
    mut audio_center: ResMut<AudioCenter>,
) {
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

    if should_reset {
        state.modified_settings.main_volume = meta.default_settings.main_volume;
        audio_center.event(state.modified_settings.volume_change_event());
    }

    ui.add_space(normal_font.size);

    ui.scope(|ui| {
        let style = ui.style_mut();
        style.drag_value_text_style = egui::TextStyle::Monospace;
        style.spacing.indent = normal_font.size / 2.5;
        style.spacing.button_padding = egui::vec2(6.0, 3.0);
        style.spacing.slider_width = 200.0;
        style.visuals.indent_has_left_vline = false;

        ui.indent("audio-controls", |ui| {
            ui.label(bigger_font.rich(localization.get("volume")));
            ui.add_space(normal_font.size / 2.0);

            egui::Grid::new("volume-grid").show(ui, |ui| {
                // Main
                let main_changed = volume_control_widget(
                    ui,
                    normal_font.rich(localization.get("volume-main")),
                    &mut state.modified_settings.main_volume,
                )
                .changed();
                ui.end_row();

                // Music
                let music_changed = volume_control_widget(
                    ui,
                    normal_font.rich(localization.get("volume-music")),
                    &mut state.modified_settings.music_volume,
                )
                .changed();
                ui.end_row();

                // Effects
                let effects_changed = volume_control_widget(
                    ui,
                    normal_font.rich(localization.get("volume-effects")),
                    &mut state.modified_settings.effects_volume,
                )
                .changed();
                ui.end_row();

                if main_changed || music_changed || effects_changed {
                    audio_center.event(state.modified_settings.volume_change_event());
                }
            });
        });
    });
}

fn volume_control_widget(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    value: &mut f64,
) -> egui::Response {
    ui.label(label);
    let slider = egui::Slider::new(value, 0.0..=1.0)
        .smallest_positive(0.0)
        .step_by(0.01)
        .fixed_decimals(3)
        .custom_formatter(|x, _| format!("{:5.1}", x * 100.0))
        .custom_parser(|s| s.parse::<f64>().ok().map(|f| f / 100.0));
    ui.add(slider)
}
