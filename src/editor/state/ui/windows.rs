use std::ops::ControlFlow;

use crate::editor::{
    actions::{UiAction, UiActionExt},
    state::State,
    windows::{CreateLayerResult, CreateTilesetResult, MenuResult, SaveMapResult},
};

impl State {
    pub(super) fn draw_windows(&mut self, egui_ctx: &egui::Context) -> Option<UiAction> {
        let mut action = None;

        if let Some(window) = &mut self.create_layer_window {
            match window.ui(egui_ctx, &self.map_resource.map) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(CreateLayerResult::Create {
                    has_collision,
                    layer_kind,
                    layer_name,
                }) => {
                    action.then_do_some(UiAction::CreateLayer {
                        has_collision,
                        kind: layer_kind,
                        index: None,
                        id: layer_name,
                    });
                    self.create_layer_window = None;
                }
                ControlFlow::Break(CreateLayerResult::Close) => {
                    self.create_layer_window = None;
                }
            }
        }

        if let Some(window) = &mut self.save_map_window {
            match window.ui(egui_ctx) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(SaveMapResult::Save { name }) => {
                    action.then_do_some(UiAction::SaveMap { name: Some(name) });
                    self.save_map_window = None;
                }
                ControlFlow::Break(SaveMapResult::Close) => {
                    self.save_map_window = None;
                }
            }
        } else if let Some(window) = &mut self.create_tileset_window {
            match window.ui(egui_ctx, &self.map_resource.map) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(CreateTilesetResult::Create {
                    tileset_name,
                    texture,
                }) => {
                    action.then_do_some(UiAction::CreateTileset {
                        id: tileset_name,
                        texture_id: texture,
                    });
                    self.create_tileset_window = None;
                }
                ControlFlow::Break(CreateTilesetResult::Close) => {
                    self.create_tileset_window = None;
                }
            }
        } else if let Some(window) = &mut self.menu_window {
            match window.ui(egui_ctx, self.map_resource.meta.is_user_map) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(MenuResult::OpenCreateMapWindow) => {
                    action.then_do_some(UiAction::OpenCreateMapWindow);
                }
                ControlFlow::Break(MenuResult::OpenLoadMapWindow) => {
                    action.then_do_some(UiAction::OpenLoadMapWindow);
                }
                ControlFlow::Break(MenuResult::OpenSaveMapWindow) => {
                    action.then_do_some(UiAction::OpenSaveMapWindow);
                }
                ControlFlow::Break(MenuResult::SaveMap) => {
                    action.then_do_some(UiAction::SaveMap { name: None });
                }
                ControlFlow::Break(MenuResult::ExitToMainMenu) => {
                    action.then_do_some(UiAction::ExitToMainMenu);
                }
                ControlFlow::Break(MenuResult::QuitToDesktop) => {
                    action.then_do_some(UiAction::QuitToDesktop);
                }
            }
        }

        action
    }
}
