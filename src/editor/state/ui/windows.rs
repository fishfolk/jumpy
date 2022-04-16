use std::ops::ControlFlow;

use crate::editor::{
    actions::UiAction,
    state::Editor,
    windows::{CreateLayerResult, CreateTilesetResult, MenuResult, SaveMapResult},
};

impl Editor {
    pub(super) fn draw_windows(&mut self, egui_ctx: &egui::Context) {
        if let Some(window) = &mut self.create_layer_window {
            match window.ui(egui_ctx, &self.map_resource.map) {
                ControlFlow::Continue(()) => (),
                ControlFlow::Break(CreateLayerResult::Create {
                    has_collision,
                    layer_kind,
                    layer_name,
                }) => {
                    self.apply_action(UiAction::CreateLayer {
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
                    self.apply_action(UiAction::SaveMap { name: Some(name) });
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
                    self.apply_action(UiAction::CreateTileset {
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
                    todo!("Create map window");
                }
                ControlFlow::Break(MenuResult::OpenLoadMapWindow) => {
                    todo!("Open/load map window")
                }
                ControlFlow::Break(MenuResult::OpenSaveMapWindow) => {
                    self.save_map_window = Some(Default::default());
                }
                ControlFlow::Break(MenuResult::SaveMap) => {
                    self.apply_action(UiAction::SaveMap { name: None });
                }
                ControlFlow::Break(MenuResult::ExitToMainMenu) => {
                    crate::exit_to_main_menu();
                }
                ControlFlow::Break(MenuResult::QuitToDesktop) => {
                    crate::quit_to_desktop();
                }
            }
        }
    }
}
