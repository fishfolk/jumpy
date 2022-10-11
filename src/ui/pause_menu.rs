use bevy_egui::*;
use bevy_fluent::Localization;
use iyes_loopless::state::NextState;

use crate::{
    ui::input::MenuAction, localization::LocalizationExt, metadata::GameMeta, prelude::*, GameState,
};

use super::widgets::{
    bordered_button::BorderedButton, bordered_frame::BorderedFrame, EguiContextExt, EguiUiExt,
};

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            unpause_system
                .run_in_state(GameState::InGame)
                .run_in_state(InGameState::Paused),
        )
        .add_system(
            pause_system
                .run_in_state(GameState::InGame)
                .run_in_state(InGameState::Playing),
        )
        .add_system(
            pause_menu
                .run_in_state(GameState::InGame)
                .run_in_state(InGameState::Paused),
        );
    }
}

/// Transition game to pause state
fn pause_system(mut commands: Commands, input: Query<&ActionState<MenuAction>>) {
    let input = input.single();
    if input.just_pressed(MenuAction::Pause) {
        commands.insert_resource(NextState(InGameState::Paused));
    }
}

// Transition game out of paused state
fn unpause_system(mut commands: Commands, input: Query<&ActionState<MenuAction>>) {
    let input = input.single();
    if input.just_pressed(MenuAction::Pause) {
        commands.insert_resource(NextState(InGameState::Playing));
    }
}

pub fn pause_menu(
    mut commands: Commands,
    mut egui_context: ResMut<EguiContext>,
    game: Res<GameMeta>,
    localization: Res<Localization>,
) {
    let ui_theme = &game.ui_theme;

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(egui_context.ctx_mut(), |ui| {
            let screen_rect = ui.max_rect();

            let pause_menu_width = game.main_menu.menu_width;
            let x_margin = (screen_rect.width() - pause_menu_width) / 2.0;
            let outer_margin = egui::style::Margin::symmetric(x_margin, screen_rect.height() * 0.2);

            BorderedFrame::new(&ui_theme.panel.border)
                .margin(outer_margin)
                .padding(ui_theme.panel.padding.into())
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    let heading_font = ui_theme
                        .font_styles
                        .heading
                        .colored(ui_theme.panel.font_color);

                    ui.vertical_centered(|ui| {
                        ui.themed_label(&heading_font, &localization.get("paused"));

                        ui.add_space(10.0);

                        let width = ui.available_width();

                        let continue_button = BorderedButton::themed(
                            &ui_theme.button_styles.normal,
                            &localization.get("continue"),
                        )
                        .min_size(egui::vec2(width, 0.0))
                        .show(ui);

                        // Focus continue button by default
                        if ui.memory().focus().is_none() {
                            continue_button.request_focus();
                        }

                        if continue_button.clicked() {
                            commands.insert_resource(NextState(GameState::InGame));
                        }

                        if BorderedButton::themed(
                            &ui_theme.button_styles.normal,
                            &localization.get("edit"),
                        )
                        .min_size(egui::vec2(width, 0.0))
                        .show(ui)
                        .clicked()
                        {
                            commands.insert_resource(NextState(InGameState::Editing))
                        }

                        if BorderedButton::themed(
                            &ui_theme.button_styles.normal,
                            &localization.get("main-menu"),
                        )
                        .min_size(egui::vec2(width, 0.0))
                        .show(ui)
                        .clicked()
                        {
                            // Show the main menu
                            commands.insert_resource(NextState(GameState::MainMenu));
                            ui.ctx().clear_focus();
                        }
                    });
                })
        });
}
