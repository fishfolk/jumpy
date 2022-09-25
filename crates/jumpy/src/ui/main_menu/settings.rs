use bevy_inspector_egui::egui::style::Margin;
use leafwing_input_manager::{axislike::SingleAxis, user_input::InputKind, Actionlike};

use super::*;

/// Render the settings menu
pub fn settings_menu_ui(
    params: &mut MenuSystemParams,
    ui: &mut egui::Ui,
    current_tab: SettingsTab,
) {
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
                        if tab == &current_tab {
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
                            *params.menu_page = MenuPage::Settings { tab: *tab };
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
                            if cancel_button.clicked() {
                                *params.menu_page = MenuPage::Main;
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
                                    params.modified_settings.as_ref().unwrap(),
                                );
                                // Persist to storage
                                params.storage.save();

                                // Go to main menu
                                *params.menu_page = MenuPage::Main;
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
                        match current_tab {
                            SettingsTab::Controls => {
                                controls_settings_ui(
                                    params,
                                    ui,
                                    // Reset button clicked
                                    bottom_buttons[1].clicked(),
                                    &tabs,
                                    &bottom_buttons,
                                )
                            }
                            SettingsTab::Sound => sound_settings_ui(ui, &params.game),
                        }
                    });
                });
            });
        });
}

// Render the control input grid
fn controls_settings_ui(
    params: &mut MenuSystemParams,
    ui: &mut egui::Ui,
    should_reset: bool,
    settings_tabs: &[egui::Response],
    bottom_buttons: &[egui::Response],
) {
    use egui_extras::Size;

    let ui_theme = &params.game.ui_theme;

    // Reset the settings when reset button is clicked
    if should_reset {
        params.modified_settings.as_mut().unwrap().player_controls =
            params.game.default_settings.player_controls.clone();
    }

    // Get the font meta for the table headings and labels
    let bigger_font = &ui_theme
        .font_styles
        .bigger
        .colored(ui_theme.panel.font_color);
    let label_font = &ui_theme
        .font_styles
        .normal
        .colored(ui_theme.panel.font_color);

    ui.add_space(bigger_font.size * 0.1);

    // Calculate the row height so that it can fit the input buttons
    let small_button_style = &ui_theme.button_styles.small;
    let row_height = small_button_style.font.size
        + small_button_style.padding.top
        + small_button_style.padding.bottom;

    // Mutably borrow the player controlls settings
    let controls = &mut params.modified_settings.as_mut().unwrap().player_controls;

    // Build the table rows of control bindings as an array of mutable InputKind's
    let mut input_rows = [
        (
            &params.localization.get("move-up"),
            [
                &mut controls.keyboard1.movement.up,
                &mut controls.keyboard2.movement.up,
                &mut controls.gamepad.movement.up,
            ],
        ),
        (
            &params.localization.get("move-down"),
            [
                &mut controls.keyboard1.movement.down,
                &mut controls.keyboard2.movement.down,
                &mut controls.gamepad.movement.down,
            ],
        ),
        (
            &params.localization.get("move-left"),
            [
                &mut controls.keyboard1.movement.left,
                &mut controls.keyboard2.movement.left,
                &mut controls.gamepad.movement.left,
            ],
        ),
        (
            &params.localization.get("move-right"),
            [
                &mut controls.keyboard1.movement.right,
                &mut controls.keyboard2.movement.right,
                &mut controls.gamepad.movement.right,
            ],
        ),
        (
            &params.localization.get("flop-attack"),
            [
                &mut controls.keyboard1.flop_attack,
                &mut controls.keyboard2.flop_attack,
                &mut controls.gamepad.flop_attack,
            ],
        ),
        (
            &params.localization.get("shoot"),
            [
                &mut controls.keyboard1.shoot,
                &mut controls.keyboard2.shoot,
                &mut controls.gamepad.shoot,
            ],
        ),
        (
            &params.localization.get("throwgrab"),
            [
                &mut controls.keyboard1.throw,
                &mut controls.keyboard2.throw,
                &mut controls.gamepad.throw,
            ],
        ),
    ];

    // Collect input button responses for building adjacency graph
    let mut input_buttons = Vec::new();

    // Create input table
    egui_extras::TableBuilder::new(ui)
        .cell_layout(egui::Layout::centered_and_justified(
            egui::Direction::LeftToRight,
        ))
        .column(Size::exact(label_font.size * 7.0))
        .column(Size::remainder())
        .column(Size::remainder())
        .column(Size::remainder())
        .header(bigger_font.size * 1.5, |mut row| {
            row.col(|ui| {
                ui.themed_label(&bigger_font, &params.localization.get("action"));
            });
            row.col(|ui| {
                ui.themed_label(&bigger_font, &params.localization.get("keyboard-1"));
            });
            row.col(|ui| {
                ui.themed_label(&bigger_font, &params.localization.get("keyboard-2"));
            });
            row.col(|ui| {
                ui.themed_label(&bigger_font, &params.localization.get("gamepad"));
            });
        })
        .body(|mut body| {
            // Keep track of the input button index we are on
            let mut input_idx = 0;

            // Loop through the input rows
            for (title, inputs) in &mut input_rows {
                body.row(row_height, |mut row| {
                    // Add row label
                    row.col(|ui| {
                        ui.themed_label(&label_font, title);
                    });

                    // Add buttons for each kind of input
                    for (button_idx, input) in inputs.iter_mut().enumerate() {
                        // The last button is a gamepad binding, the others are keyboard
                        let binding_kind = if button_idx == 2 {
                            BindingKind::Gamepad
                        } else {
                            BindingKind::Keyboard
                        };

                        // Render the button
                        row.col(|ui| {
                            let button = BorderedButton::themed(
                                &ui_theme.button_styles.small,
                                format_input(input),
                            )
                            .show(ui);

                            // Start an input binding if the button is clicked
                            if button.clicked() {
                                *params.currently_binding_input_idx = Some(input_idx);
                            }

                            // If we are binding an input for this button
                            if *params.currently_binding_input_idx == Some(input_idx) {
                                // Render the binding window
                                egui::Window::new("input_binding_overlay")
                                    .auto_sized()
                                    .collapsible(false)
                                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                                    .frame(egui::Frame::none())
                                    .title_bar(false)
                                    .show(ui.ctx(), |ui| {
                                        let font = params
                                            .game
                                            .ui_theme
                                            .font_styles
                                            .normal
                                            .colored(params.game.ui_theme.panel.font_color);

                                        let border = &params.game.ui_theme.panel.border;
                                        let m = &border.border_size;
                                        let s = border.scale;
                                        BorderedFrame::new(border)
                                            // Just enough padding to fit the frame's border image
                                            .padding(Margin {
                                                left: m.left * s,
                                                right: m.right * s,
                                                top: m.top * s,
                                                bottom: m.bottom * s,
                                            })
                                            .show(ui, |ui| {
                                                ui.themed_label(
                                                    &font,
                                                    &params.localization.get("bind-input"),
                                                );

                                                // See if there has been any inputs of the kind we
                                                // are binding.
                                                let get_input =
                                                    params.control_inputs.get_event(binding_kind);

                                                // If there has been an input
                                                if let Ok(Some(input_kind)) = get_input {
                                                    // Stop listening for inputs
                                                    *params.currently_binding_input_idx = None;

                                                    // Reset the focus on the input button
                                                    button.request_focus();

                                                    // Set the input for this button to the pressed
                                                    // input
                                                    **input = input_kind;

                                                // If the user cancelled the input binding
                                                } else if get_input.is_err() {
                                                    // Set the focus back on the button
                                                    button.request_focus();
                                                    // And stop listening for inputs
                                                    *params.currently_binding_input_idx = None;
                                                }

                                                // Make sure we don't double-trigger any menu
                                                // actions while on this menu by consuming all menu
                                                // actions.
                                                let mut menu_input = params.menu_input.single_mut();
                                                for action in MenuAction::variants() {
                                                    menu_input.consume(action);
                                                }
                                            });
                                    });
                            }

                            // Add input button to the list
                            input_buttons.push(button);
                        });

                        // Increment the input button index
                        input_idx += 1;
                    }
                });
            }
        });

    // Set adjacency for all of the gamepad input buttons
    for row_idx in 0..input_rows.len() {
        if row_idx == 0 {
            // Reverse button order here so that the first input button gets priority when
            // navigating down from the tabs.
            for i in (0..3).rev() {
                let button = &input_buttons[row_idx * 3 + i];

                // The top row of buttons is below the settings tabs
                for tab in settings_tabs {
                    params.adjacencies.widget(tab).above(button);
                }

                // The last tab is considered to the left of the first button
                if i == 0 {
                    params
                        .adjacencies
                        .widget(button)
                        .to_right_of(&settings_tabs[settings_tabs.len() - 1]);
                }
            }

        // If this is the last row, the input buttons are above the bottom buttons
        } else if row_idx == input_rows.len() - 1 {
            for i in 0..3 {
                let button_above = &input_buttons[(row_idx - 1) * 3 + i];
                let button = &input_buttons[row_idx * 3 + i];

                params
                    .adjacencies
                    .widget(button)
                    .above(&bottom_buttons[i])
                    .below(button_above);

                // The first bottom button is to the right of the last input button
                if i == 2 {
                    params
                        .adjacencies
                        .widget(button)
                        .to_left_of(&bottom_buttons[0]);
                }
            }

        // If this is a middle row, set the input buttons to be below the ones in the row above
        } else {
            for i in 0..3 {
                let button_above = &input_buttons[(row_idx - 1) * 3 + i];
                let button = &input_buttons[row_idx * 3 + i];

                params.adjacencies.widget(button).below(button_above);
            }
        }
    }
}

/// Render the sound settings UI
fn sound_settings_ui(_ui: &mut egui::Ui, _game: &GameMeta) {
    // This is un-reachable right now
    todo!("Implement sound settings UI");
}

/// Format an InputKind as a user-facing string
fn format_input(input: &InputKind) -> String {
    match input {
        InputKind::SingleAxis(axis) => {
            // If we set the positive low to 1.0, then that means we don't trigger on positive
            // movement, and it must be a negative movement binding.
            let direction = if axis.positive_low == 1.0 { "-" } else { "+" };

            let stick = match axis.axis_type {
                leafwing_input_manager::axislike::AxisType::Gamepad(axis) => {
                    format!("{:?}", axis)
                }
                other => format!("{:?}", other),
            };

            format!("{} {}", stick, direction)
        }
        other => other.to_string(),
    }
}

/// Helper system param to get input events that we are interested in for input binding.
#[derive(SystemParam)]
pub struct ControlInputBindingEvents<'w, 's> {
    keys: Res<'w, Input<KeyCode>>,
    gamepad_buttons: Res<'w, Input<GamepadButton>>,
    gamepad_events: EventReader<'w, 's, GamepadEvent>,
}

/// The kind of input binding to listen for.
enum BindingKind {
    Keyboard,
    Gamepad,
}

impl<'w, 's> ControlInputBindingEvents<'w, 's> {
    // Get the next event, if any
    fn get_event(&mut self, binding_kind: BindingKind) -> Result<Option<InputKind>, ()> {
        if self.keys.just_pressed(KeyCode::Escape) {
            return Err(());
        }

        Ok(match binding_kind {
            // If we are looking for keyboard inputs
            BindingKind::Keyboard => {
                // Just return the first pressed button we find
                if let Some(&key_code) = self.keys.get_just_pressed().next() {
                    Some(key_code.into())
                } else {
                    None
                }
            }
            // If we are looking for gamepad events
            BindingKind::Gamepad => {
                // Return the first pressed button we find, if any
                if let Some(&button) = self.gamepad_buttons.get_just_pressed().next() {
                    Some(button.button_type.into())

                // If we can't find a button pressed
                } else {
                    // Look for axes tilted more than 0.5 in either direction.
                    for gamepad_event in self.gamepad_events.iter() {
                        if let GamepadEventType::AxisChanged(axis, value) = gamepad_event.event_type
                        {
                            // Create an axis positive movement binding
                            if value > 0.5 {
                                return Ok(Some(
                                    SingleAxis {
                                        axis_type: axis.into(),
                                        positive_low: 0.1,
                                        negative_low: -1.0,
                                        value: None,
                                    }
                                    .into(),
                                ));
                            // Create an axis negative movement binding
                            } else if value < -0.5 {
                                return Ok(Some(
                                    SingleAxis {
                                        axis_type: axis.into(),
                                        positive_low: 1.0,
                                        negative_low: -0.1,
                                        value: None,
                                    }
                                    .into(),
                                ));
                            }
                        }
                    }

                    None
                }
            }
        })
    }
}
