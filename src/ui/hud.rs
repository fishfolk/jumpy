//! In-game HUD

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::{
    damage::Health,
    fighter::Inventory,
    metadata::{FighterMeta, GameMeta},
    player::PlayerIndex,
    ui::widgets::{bordered_frame::BorderedFrame, progress_bar::ProgressBar, EguiUIExt},
    Player, Stats,
};

pub fn render_hud(
    mut egui_context: ResMut<EguiContext>,
    players: Query<
        (
            &PlayerIndex,
            &Stats,
            &Health,
            &Handle<FighterMeta>,
            &Inventory,
        ),
        With<Player>,
    >,
    game: Res<GameMeta>,
    fighter_assets: Res<Assets<FighterMeta>>,
) {
    let ui_theme = &game.ui_theme;

    // Helper struct for holding player hud info
    struct PlayerInfo {
        name: String,
        life: f32,
        portrait_texture_id: egui::TextureId,
        portrait_size: egui::Vec2,
        item: Option<ItemInfo>,
    }

    struct ItemInfo {
        texture_id: egui::TextureId,
        size: egui::Vec2,
    }

    // Collect player info
    let mut players = players.iter().collect::<Vec<_>>();
    players.sort_by_key(|(player_i, _, _, _, _)| player_i.0);

    let player_infos = players
        .into_iter()
        .filter_map(|(_, stats, health, fighter_handle, inventory)| {
            fighter_assets.get(fighter_handle).map(|fighter| {
                let portrait_size = fighter.hud.portrait.image_size;
                PlayerInfo {
                    name: fighter.name.clone(),
                    life: **health as f32 / stats.max_health as f32,
                    portrait_texture_id: egui_context
                        .add_image(fighter.hud.portrait.image_handle.clone_weak()),
                    portrait_size: egui::Vec2::new(portrait_size.x, portrait_size.y),
                    item: inventory.as_ref().map(|item_meta| ItemInfo {
                        texture_id: egui_context
                            .add_image(item_meta.image.image_handle.clone_weak()),
                        size: egui::Vec2::new(
                            item_meta.image.image_size.x,
                            item_meta.image.image_size.y,
                        ),
                    }),
                }
            })
        })
        .collect::<Vec<_>>();

    let border = ui_theme.hud.portrait_frame.border_size;
    let scale = ui_theme.hud.portrait_frame.scale;
    let portrait_frame_padding = egui::style::Margin {
        left: border.left * scale,
        right: border.right * scale,
        top: border.top * scale,
        bottom: border.bottom * scale,
    };

    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(egui_context.ctx_mut(), |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                for player in player_infos {
                    ui.add_space(20.0);

                    ui.vertical(|ui| {
                        ui.allocate_ui(egui::Vec2::new(ui_theme.hud.player_hud_width, 50.), |ui| {
                            ui.themed_label(&ui_theme.hud.font, &player.name);

                            ui.horizontal(|ui| {
                                BorderedFrame::new(&ui_theme.hud.portrait_frame)
                                    .padding(portrait_frame_padding)
                                    .show(ui, |ui| {
                                        ui.image(player.portrait_texture_id, player.portrait_size);
                                    });

                                ui.vertical(|ui| {
                                    ui.add_space(5.0);
                                    ProgressBar::new(&ui_theme.hud.lifebar, player.life)
                                        .min_width(ui.available_width())
                                        .show(ui);

                                    ui.vertical(|ui| {
                                        if let Some(item) = player.item {
                                            ui.add_space(5.0);
                                            ui.image(item.texture_id, item.size);
                                        }
                                    });
                                });
                            });
                        });
                    });
                }
            });
        });
}
