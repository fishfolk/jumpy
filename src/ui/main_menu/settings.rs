use crate::{
    prelude::*,
    settings::{PlayerControlMapping, Settings},
};

use super::MenuPage;

mod audio;
mod controls;
mod graphics;
mod networking;

#[derive(Clone, Default)]
struct SettingsState {
    tab: SettingsTab,
    modified_settings: Settings,
    settings_loaded: bool,
    currently_binding_input_idx: Option<usize>,
}

/// Which settings tab we are on
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum SettingsTab {
    #[default]
    Controls,
    Networking,
    Audio,
    Graphics,
}

impl SettingsTab {
    const TABS: &'static [(Self, &'static str)] = &[
        (Self::Controls, "controls"),
        (Self::Networking, "networking"),
        (Self::Audio, "audio"),
        (Self::Graphics, "graphics"),
    ];
}

pub fn widget(
    mut ui: In<&mut egui::Ui>,
    meta: Root<GameMeta>,
    localization: Localization<GameMeta>,
    input: Res<GlobalPlayerControls>,
    mut mapping: ResMut<PlayerControlMapping>,
    mut storage: ResMut<Storage>,
    world: &World,
) {
    let ui = &mut *ui;
    let mut state = ui.ctx().get_state::<SettingsState>();
    let screen_rect = ui.max_rect();

    if !state.settings_loaded {
        state.modified_settings = storage.get::<Settings>().unwrap().clone();
        state.settings_loaded = true;
    }

    let mut should_reset = false;

    // Calculate a margin
    let outer_margin = screen_rect.size() * 0.10;
    let outer_margin = egui::Margin {
        left: outer_margin.x,
        right: outer_margin.x,
        // Make top and bottom margins smaller
        top: outer_margin.y / 1.5,
        bottom: outer_margin.y / 1.5,
    };

    let heading_font = meta
        .theme
        .font_styles
        .heading
        .with_color(meta.theme.panel.font_color);

    BorderedFrame::new(&meta.theme.panel.border)
        .margin(outer_margin)
        .padding(meta.theme.panel.padding)
        .show(ui, |ui| {
            // Disable all the buttons if we are currently binding an input
            ui.set_enabled(state.currently_binding_input_idx.is_none());

            ui.vertical_centered(|ui| {
                // Settings Heading
                ui.label(heading_font.rich(localization.get("settings")));

                // Add tab list at the top of the panel
                ui.horizontal(|ui| {
                    for (i, (tab, name)) in SettingsTab::TABS.iter().enumerate() {
                        let name = localization.get(name);
                        let mut name = egui::RichText::new(name);

                        // Underline the current tab
                        if *tab == state.tab {
                            name = name.underline();
                        }

                        let mut button =
                            BorderedButton::themed(&meta.theme.buttons.normal, name).show(ui);

                        // Focus the first tab by default
                        if i == 0 {
                            button = button.focus_by_default(ui);
                        }

                        // Change tab when clicked
                        if button.clicked() {
                            state.tab = *tab;
                        }
                    }
                });

                // Add buttons to the bottom
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    let buttons_response = ui.horizontal(|ui| {
                        // Calculate button size and spacing
                        let width = ui.available_width();
                        let button_width = width / 4.0;
                        let button_min_size = vec2(button_width, 0.0);
                        let button_spacing = (width - 3.0 * button_width) / 4.0;

                        ui.add_space(button_spacing);

                        // Cancel button
                        if (BorderedButton::themed(
                            &meta.theme.buttons.normal,
                            localization.get("cancel"),
                        )
                        .min_size(button_min_size)
                        .show(ui)
                        .clicked()
                            || input.values().any(|x| x.menu_back_just_pressed))
                            && state.currently_binding_input_idx.is_none()
                        {
                            ui.ctx().set_state(MenuPage::Home);
                            // Reset the modified settings to their stored settings.
                            state.modified_settings = storage.get::<Settings>().unwrap().clone();
                            // Run cancel callbacks
                            if state.tab == SettingsTab::Audio {
                                world.run_system(audio::on_cancel, &state);
                            }
                        }

                        ui.add_space(button_spacing);

                        // Reset button
                        if BorderedButton::themed(
                            &meta.theme.buttons.normal,
                            localization.get("reset"),
                        )
                        .min_size(button_min_size)
                        .show(ui)
                        .clicked()
                        {
                            should_reset = true;
                        }

                        ui.add_space(button_spacing);

                        // Save button
                        if BorderedButton::themed(
                            &meta.theme.buttons.normal,
                            localization.get("save"),
                        )
                        .min_size(button_min_size)
                        .show(ui)
                        .clicked()
                        {
                            // Save the settings to disk.
                            *mapping = state.modified_settings.player_controls.clone();
                            storage.insert(state.modified_settings.clone());
                            storage.save();
                            ui.ctx().set_state(MenuPage::Home);
                        }
                    });

                    let buttons_rect = buttons_response.response.rect;
                    ui.add_space(buttons_rect.height() / 2.0);

                    ui.vertical(|ui| match state.tab {
                        SettingsTab::Controls => {
                            world.run_system(controls::widget, (ui, &mut state, should_reset))
                        }
                        SettingsTab::Networking => {
                            world.run_system(networking::widget, (ui, &mut state, should_reset))
                        }
                        SettingsTab::Audio => {
                            world.run_system(audio::widget, (ui, &mut state, should_reset))
                        }
                        SettingsTab::Graphics => {
                            world.run_system(graphics::widget, (ui, &mut state, should_reset))
                        }
                    });
                });
            });
        });

    // Update the state with our modified one
    ui.ctx().set_state(state);
}
