use std::marker::PhantomData;

use bevy::ecs::system::SystemParam;
use bevy_egui::*;
use bevy_fluent::Localization;

use crate::{localization::LocalizationExt, prelude::*};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            editor
                .run_in_state(GameState::InGame)
                .run_in_state(InGameState::Editing),
        );
    }
}

#[derive(SystemParam)]
pub struct EditorParams<'w, 's> {
    commands: Commands<'w, 's>,
    localization: Res<'w, Localization>,
    #[system_param(ignore)]
    _phantom: PhantomData<&'s ()>,
}

/// The map editor system
pub fn editor(mut params: EditorParams, mut egui_ctx: ResMut<EguiContext>) {
    let ctx = egui_ctx.ctx_mut();
    egui::TopBottomPanel::top("top-bar").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            ui.label(&params.localization.get("editor"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(&params.localization.get("main-menu")).clicked() {
                    params
                        .commands
                        .insert_resource(NextState(GameState::MainMenu));
                }
            });
        });
    });
}
