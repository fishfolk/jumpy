use bevy_inspector_egui::egui::style::Margin;
use leafwing_input_manager::{axislike::SingleAxis, user_input::InputKind, Actionlike};

use super::*;

mod controls;
mod networking;
mod sound;

/// Which settings tab we are on
#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Controls,
    #[allow(unused)] // TODO: Just for now until we get sound settings setup
    Sound,
    Networking,
}

impl Default for SettingsTab {
    fn default() -> Self {
        Self::Controls
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ModifiedSettings(pub Option<Settings>);

#[derive(SystemParam)]
pub struct SettingsMenu<'w, 's> {
    game: Res<'w, GameMeta>,
    current_tab: ResMut<'w, SettingsTab>,
    menu_page: ResMut<'w, MenuPage>,
    modified_settings: ResMut<'w, ModifiedSettings>,
    currently_binding_input_idx: Local<'s, Option<usize>>,
    localization: Res<'w, Localization>,
    adjacencies: ResMut<'w, WidgetAdjacencies>,
    storage: ResMut<'w, Storage>,
    control_inputs: controls::ControlInputBindingEvents<'w, 's>,
    menu_input: Query<'w, 's, &'static mut ActionState<MenuAction>>,
    #[system_param(ignore)]
    _phantom: PhantomData<(&'w (), &'s ())>,
}

impl<'w, 's> WidgetSystem for SettingsMenu<'w, 's> {
    type Args = ();

    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut egui::Ui,
        _id: WidgetId,
        _args: Self::Args,
    ) {
        let mut params: SettingsMenu = state.get_mut(world);

        let screen_rect = ui.max_rect();

        // Calculate a margin
        let outer_margin = screen_rect.size() * 0.10;
        let outer_margin = Margin {
            left: outer_margin.x,
            right: outer_margin.x,
            // Make top and bottom margins smaller
            top: outer_margin.y / 1.5,
            bottom: outer_margin.y / 1.5,
        };

        BorderedFrame::new(&params.game.ui_theme.panel.border)
            .margin(outer_margin)
            .padding(params.game.ui_theme.panel.padding.into())
            .show(ui, |ui| {
                // Disable all the buttons if we are currently binding an input
                ui.set_enabled(params.currently_binding_input_idx.is_none());

                ui.vertical_centered(|ui| {
                    // Settings Heading
                    ui.themed_label(
                        &params.game.ui_theme.font_styles.heading,
                        &params.localization.get("settings"),
                    );

                    // Add tab list at the top of the panel
                    let mut tabs = Vec::new();
                    ui.horizontal(|ui| {
                        for (i, (tab, name)) in SettingsTab::TABS.iter().enumerate() {
                            let name = &params.localization.get(*name);
                            let mut name = egui::RichText::new(name);

                            // Underline the current tab
                            if tab == &*params.current_tab {
                                name = name.underline();
                            }

                            let mut button = BorderedButton::themed(
                                &params.game.ui_theme.button_styles.normal,
                                name,
                            )
                            .show(ui);

                            // Focus the first tab by default
                            if i == 0 {
                                button = button.focus_by_default(ui);
                            }

                            // Change tab when clicked
                            if button.clicked() {
                                *params.current_tab = *tab;
                            }

                            tabs.push(button);
                        }
                    });

                    // Add buttons to the bottom
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        // Collect button responses so we can add them to adjacency map later
                        let bottom_buttons = ui
                            .horizontal(|ui| {
                                // Calculate button size and spacing
                                let width = ui.available_width();
                                let button_width = width / 4.0;
                                let button_min_size = egui::vec2(button_width, 0.0);
                                let button_spacing = (width - 3.0 * button_width) / 4.0;

                                ui.add_space(button_spacing);

                                // Cancel button
                                let cancel_button = BorderedButton::themed(
                                    &params.game.ui_theme.button_styles.normal,
                                    &params.localization.get("cancel"),
                                )
                                .min_size(button_min_size)
                                .show(ui);

                                // Go to menu when cancel is clicked
                                if cancel_button.clicked()
                                    || params.menu_input.single().just_pressed(MenuAction::Back)
                                {
                                    *params.menu_page = MenuPage::Home;
                                    ui.ctx().clear_focus();
                                }

                                ui.add_space(button_spacing);

                                // Reset button
                                let reset_button = BorderedButton::themed(
                                    &params.game.ui_theme.button_styles.normal,
                                    &params.localization.get("reset"),
                                )
                                .min_size(button_min_size)
                                .show(ui);

                                ui.add_space(button_spacing);

                                // Save button
                                let save_button = BorderedButton::themed(
                                    &params.game.ui_theme.button_styles.normal,
                                    &params.localization.get("save"),
                                )
                                .min_size(button_min_size)
                                .show(ui);

                                // Save new settings if settings button clicked
                                if save_button.clicked() {
                                    // Update in-memory settings
                                    params.storage.set(
                                        Settings::STORAGE_KEY,
                                        params.modified_settings.0.as_ref().unwrap(),
                                    );
                                    // Persist to storage
                                    params.storage.save();

                                    // Go to main menu
                                    *params.menu_page = MenuPage::Home;
                                    ui.ctx().clear_focus();
                                }

                                [cancel_button, reset_button, save_button]
                            })
                            .inner;

                        // Set the last bottom button's adjacency to be to the "left" of the first tab. This way
                        // you can wrap around from the bottom right button to the top left.
                        params
                            .adjacencies
                            .widget(&bottom_buttons[bottom_buttons.len() - 1])
                            .to_left_of(&tabs[0]);

                        ui.vertical(|ui| {
                            // Render selected tab
                            match *params.current_tab {
                                SettingsTab::Controls => {
                                    controls::controls_settings_ui(
                                        &mut params,
                                        ui,
                                        // Reset button clicked
                                        bottom_buttons[1].clicked(),
                                        &tabs,
                                        &bottom_buttons,
                                    )
                                }
                                SettingsTab::Networking => networking::networking_settings_ui(
                                    &mut params,
                                    ui,
                                    bottom_buttons[1].clicked(),
                                    &tabs,
                                    &bottom_buttons,
                                ),
                                SettingsTab::Sound => sound::sound_settings_ui(ui, &params.game),
                            }
                        });
                    });
                });
            });
    }
}
