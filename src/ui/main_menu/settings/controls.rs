use egui_extras::{Column, TableBuilder};

use crate::settings::InputKind;

use super::*;

pub(super) fn widget(
    mut args: In<(&mut egui::Ui, &mut SettingsState, bool)>,
    meta: Root<GameMeta>,
    localization: Localization<GameMeta>,
    keyboard: Res<KeyboardInputs>,
    gamepad: Res<GamepadInputs>,
) {
    let (ui, state, should_reset) = &mut *args;

    if *should_reset {
        state.modified_settings.player_controls = meta.default_settings.player_controls.clone();
    }

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

    ui.add_space(bigger_font.size * 0.1);

    // Calculate the row height so that it can fit the input buttons
    let small_button_style = &meta.theme.buttons.small;
    let row_height = small_button_style.font.size
        + small_button_style.padding.top
        + small_button_style.padding.bottom;

    let mapping = &mut state.modified_settings.player_controls;

    let mut input_rows = [
        (
            localization.get("move-up"),
            [
                &mut mapping.keyboard1.movement.up,
                &mut mapping.keyboard2.movement.up,
                &mut mapping.gamepad.movement.up,
            ],
        ),
        (
            localization.get("move-down"),
            [
                &mut mapping.keyboard1.movement.down,
                &mut mapping.keyboard2.movement.down,
                &mut mapping.gamepad.movement.down,
            ],
        ),
        (
            localization.get("move-left"),
            [
                &mut mapping.keyboard1.movement.left,
                &mut mapping.keyboard2.movement.left,
                &mut mapping.gamepad.movement.left,
            ],
        ),
        (
            localization.get("move-right"),
            [
                &mut mapping.keyboard1.movement.right,
                &mut mapping.keyboard2.movement.right,
                &mut mapping.gamepad.movement.right,
            ],
        ),
        (
            localization.get("jump"),
            [
                &mut mapping.keyboard1.jump,
                &mut mapping.keyboard2.jump,
                &mut mapping.gamepad.jump,
            ],
        ),
        (
            localization.get("grab-drop"),
            [
                &mut mapping.keyboard1.grab,
                &mut mapping.keyboard2.grab,
                &mut mapping.gamepad.grab,
            ],
        ),
        (
            localization.get("shoot"),
            [
                &mut mapping.keyboard1.shoot,
                &mut mapping.keyboard2.shoot,
                &mut mapping.gamepad.shoot,
            ],
        ),
        (
            localization.get("slide"),
            [
                &mut mapping.keyboard1.slide,
                &mut mapping.keyboard2.slide,
                &mut mapping.gamepad.slide,
            ],
        ),
        (
            localization.get("pause"),
            [
                &mut mapping.keyboard1.pause,
                &mut mapping.keyboard2.pause,
                &mut mapping.gamepad.pause,
            ],
        ),
        (
            localization.get("menu-confirm"),
            [
                &mut mapping.keyboard1.menu_confirm,
                &mut mapping.keyboard2.menu_confirm,
                &mut mapping.gamepad.menu_confirm,
            ],
        ),
        (
            localization.get("menu-back"),
            [
                &mut mapping.keyboard1.menu_back,
                &mut mapping.keyboard2.menu_back,
                &mut mapping.gamepad.menu_back,
            ],
        ),
        (
            localization.get("menu-start"),
            [
                &mut mapping.keyboard1.menu_start,
                &mut mapping.keyboard2.menu_start,
                &mut mapping.gamepad.menu_start,
            ],
        ),
    ];

    // Create input table
    let width = ui.available_width();
    let label_size = normal_font.size * 7.0;
    let remaining_width = width - label_size;
    let cell_width = remaining_width / 3.1;
    TableBuilder::new(ui)
        .cell_layout(egui::Layout::centered_and_justified(
            egui::Direction::LeftToRight,
        ))
        .column(Column::exact(label_size))
        .column(Column::exact(cell_width))
        .column(Column::exact(cell_width))
        .column(Column::exact(cell_width))
        .header(bigger_font.size * 1.5, |mut row| {
            row.col(|ui| {
                ui.label(bigger_font.rich(localization.get("action")));
            });
            row.col(|ui| {
                ui.label(bigger_font.rich(localization.get("keyboard-1")));
            });
            row.col(|ui| {
                ui.label(bigger_font.rich(localization.get("keyboard-2")));
            });
            row.col(|ui| {
                ui.label(bigger_font.rich(localization.get("gamepad")));
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
                        ui.label(normal_font.rich(title.to_string()));
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
                            ui.set_width(ui.available_width() * 0.92);

                            let button = BorderedButton::themed(
                                &meta.theme.buttons.small,
                                input.to_string(),
                            )
                            .show(ui);

                            // Start an input binding if the button is clicked
                            if button.clicked() {
                                state.currently_binding_input_idx = Some(input_idx);
                            }
                            // If we are binding an input for this button
                            else if state.currently_binding_input_idx == Some(input_idx) {
                                // Render the binding window
                                egui::Window::new("input_binding_overlay")
                                    .auto_sized()
                                    .collapsible(false)
                                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                                    .frame(egui::Frame::none())
                                    .title_bar(false)
                                    .show(ui.ctx(), |ui| {
                                        let m = meta.theme.panel.border.border_size;
                                        let s = meta.theme.panel.border.scale;
                                        BorderedFrame::new(&meta.theme.panel.border)
                                            // Just enough padding to fit the frame's border image
                                            .padding(egui::Margin {
                                                left: m.left * s,
                                                right: m.right * s,
                                                top: m.top * s,
                                                bottom: m.bottom * s,
                                            })
                                            .show(ui, |ui| {
                                                ui.label(normal_font.rich(localization.get_with(
                                                    "bind-input",
                                                    &fluent_args! {
                                                        "binding" => title.as_ref(),
                                                        "binding_kind" => match binding_kind {
                                                            BindingKind::Keyboard => "keyboard",
                                                            BindingKind::Gamepad => "gamepad",
                                                        }
                                                    },
                                                )));

                                                ui.add_space(normal_font.size / 2.0);

                                                ui.horizontal(|ui| {
                                                    // Cancel button
                                                    if BorderedButton::themed(
                                                        &meta.theme.buttons.small,
                                                        localization.get("cancel"),
                                                    )
                                                    .show(ui)
                                                    .clicked()
                                                    {
                                                        button.request_focus();
                                                        state.currently_binding_input_idx = None;
                                                    }

                                                    // Clear button
                                                    if BorderedButton::themed(
                                                        &meta.theme.buttons.small,
                                                        localization.get("clear-binding"),
                                                    )
                                                    .show(ui)
                                                    .clicked()
                                                    {
                                                        **input = InputKind::None;
                                                        button.request_focus();
                                                        state.currently_binding_input_idx = None;
                                                    }
                                                });

                                                // See if there has been any inputs of the kind we
                                                // are binding.
                                                let bound_input =
                                                    get_input(binding_kind, &keyboard, &gamepad);

                                                // If there has been an input
                                                if let Some(input_kind) = bound_input {
                                                    // Stop listening for inputs
                                                    state.currently_binding_input_idx = None;

                                                    // Reset the focus on the input button
                                                    button.request_focus();

                                                    // Set the input for this button to the pressed
                                                    // input
                                                    **input = input_kind;
                                                }
                                            });
                                    });
                            }
                        });

                        // Increment the input button index
                        input_idx += 1;
                    }
                });
            }
        });
}

/// The kind of input binding to listen for.
enum BindingKind {
    Keyboard,
    Gamepad,
}

fn get_input(
    kind: BindingKind,
    keyboard: &KeyboardInputs,
    gamepad: &GamepadInputs,
) -> Option<InputKind> {
    match kind {
        BindingKind::Keyboard => keyboard.key_events.iter().next().and_then(|event| {
            event
                .button_state
                .pressed()
                .then(|| event.key_code.option().map(InputKind::Keyboard))
                .flatten()
        }),
        BindingKind::Gamepad => {
            gamepad
                .gamepad_events
                .iter()
                .next()
                .and_then(|event| match event {
                    GamepadEvent::Button(e) => {
                        (e.value.abs() > 0.1).then(|| InputKind::Button(e.button.clone()))
                    }
                    GamepadEvent::Axis(e) => {
                        if e.value > 0.1 {
                            Some(InputKind::AxisPositive(e.axis.clone()))
                        } else if e.value < -0.1 {
                            Some(InputKind::AxisNegative(e.axis.clone()))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
        }
    }
}
