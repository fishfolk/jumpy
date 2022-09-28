use std::marker::PhantomData;

use bevy::ecs::system::SystemParam;
use bevy_egui::*;
use bevy_fluent::Localization;

use crate::{localization::LocalizationExt, metadata::GameMeta, prelude::*};

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
    game: Res<'w, GameMeta>,
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

    egui::SidePanel::left("left-toolbar")
        .width_range(40.0..=40.0)
        .resizable(false)
        .show(ctx, |ui| {
            let icons = &params.game.ui_theme.editor.icons;
            let width = ui.available_width();
            for image in &[&icons.select, &icons.tile, &icons.spawn, &icons.erase] {
                ui.add_space(ui.spacing().window_margin.top);

                let image_aspect = image.image_size.y / image.image_size.x;
                let height = width * image_aspect;
                ui.add(egui::ImageButton::new(
                    image.egui_texture_id,
                    egui::vec2(width, height),
                ));
            }
        });

    egui::SidePanel::right("right-toolbar")
        .min_width(190.0)
        .show(ctx, |ui| {
            ui.add_space(ui.spacing().window_margin.top);
            ui.horizontal(|ui| {
                ui.label("Layers");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("➕")
                        .on_hover_text("Create a new metatile")
                        .clicked()
                    {}
                });
            });
            ui.separator();
        });
}
