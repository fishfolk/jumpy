use egui_extras::Column;

use super::*;

// Render the control input grid
pub fn controls_settings_ui(
    params: &mut SettingsMenu,
    ui: &mut egui::Ui,
    should_reset: bool,
    settings_tabs: &[egui::Response],
    bottom_buttons: &[egui::Response],
) {
    let ui_theme = &params.game.ui_theme;

    // Reset the settings when reset button is clicked
    if should_reset {
        params.modified_settings.0.as_mut().unwrap().player_controls =
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
    let controls = &mut params.modified_settings.0.as_mut().unwrap().player_controls;

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
            &params.localization.get("jump"),
            [
                &mut controls.keyboard1.jump,
                &mut controls.keyboard2.jump,
                &mut controls.gamepad.jump,
            ],
        ),
        (
            &params.localization.get("grab-drop"),
            [
                &mut controls.keyboard1.grab,
                &mut controls.keyboard2.grab,
                &mut controls.gamepad.grab,
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
            &params.localization.get("slide"),
            [
                &mut controls.keyboard1.slide,
                &mut controls.keyboard2.slide,
                &mut controls.gamepad.slide,
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
        .column(Column::exact(label_font.size * 7.0))
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::remainder())
        .header(bigger_font.size * 1.5, |mut row| {
            row.col(|ui| {
                ui.themed_label(bigger_font, &params.localization.get("action"));
            });
            row.col(|ui| {
                ui.themed_label(bigger_font, &params.localization.get("keyboard-1"));
            });
            row.col(|ui| {
                ui.themed_label(bigger_font, &params.localization.get("keyboard-2"));
            });
            row.col(|ui| {
                ui.themed_label(bigger_font, &params.localization.get("gamepad"));
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
                        ui.themed_label(label_font, title);
                    });

                    // Add buttons for each kind of input
                    for (button_idx, input) in inputs.iter_mut().enumerate() {
                        // The last button is a gamepad binding, the others are keyboard
                        let binding_kind = if button_idx == 2 {
                            BindingKind::Gamepad
                        } else {
                            BindingKind::KeyboardMouse
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

/// Format an InputKind as a user-facing string
fn format_input(input: &InputKind) -> String {
    match input {
        InputKind::SingleAxis(axis) => {
            // If we set the positive low to 1.0, then that means we don't trigger on positive
            // movement, and it must be a negative movement binding.
            let direction = if axis.positive_low == 1.0 { "-" } else { "+" };

            let stick = match axis.axis_type {
                leafwing_input_manager::axislike::AxisType::Gamepad(axis) => {
                    format!("{axis:?}")
                }
                other => format!("{other:?}"),
            };

            format!("{stick} {direction}")
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
    mouse: Res<'w, Input<MouseButton>>,
}

/// The kind of input binding to listen for.
enum BindingKind {
    KeyboardMouse,
    Gamepad,
}

impl<'w, 's> ControlInputBindingEvents<'w, 's> {
    // Get the next event, if any
    fn get_event(&mut self, binding_kind: BindingKind) -> Result<Option<InputKind>, ()> {
        if self.keys.just_pressed(KeyCode::Escape) {
            return Err(());
        }

        Ok(match binding_kind {
            // If we are looking for keyboard or mouse inputs
            BindingKind::KeyboardMouse => {
                // Check for mouse input first
                if let Some(&mouse_button) = self.mouse.get_just_pressed().next() {
                    return Ok(Some(mouse_button.into()));
                }
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
