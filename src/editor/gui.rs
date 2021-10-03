pub mod windows;
pub mod toolbars;
pub mod context_menu;

pub mod skins;

pub use skins::EditorSkinCollection;

use macroquad::{
    experimental::{
        collections::storage,
    },
    ui::{
        widgets,
        root_ui,
    },
    hash,
    prelude::*,
};

use super::{
    EditorAction,
};

use crate::{
    map::{
        Map,
    },
    gui::GuiResources,
};

use toolbars::{
    ToolbarPosition,
    Toolbar,
    ToolbarElement,
    ToolbarElementParams,
    LayerListElement,
    TilesetListElement,
    TilesetDetailsElement,
};

use windows::{
    Window,
    WindowParams,
    WindowResult,
};

pub use windows::{
    ConfirmDialog,
    CreateLayerWindow,
    CreateTilesetWindow,
};

use context_menu::{
    ContextMenu,
    ContextMenuEntry,
};

pub const NO_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.0);

pub const ELEMENT_MARGIN: f32 = 8.0;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GuiElement {
    ContextMenu,
    Toolbar,
    Window,
}

#[derive(Debug, Clone)]
pub struct EditorDrawParams {
    pub selected_layer: Option<String>,
    pub selected_tileset: Option<String>,
    pub selected_tile: Option<u32>,
}

pub struct EditorGui {
    left_toolbar: Toolbar,
    right_toolbar: Toolbar,
    open_windows: Vec<Box<dyn Window>>,
    context_menu: Option<ContextMenu>,
}

impl EditorGui {
    const LEFT_TOOLBAR_WIDTH: f32 = 50.0;
    const RIGHT_TOOLBAR_WIDTH: f32 = 250.0;

    const LAYER_LIST_HEIGHT_FACTOR: f32 = 0.2;
    const TILESET_LIST_HEIGHT_FACTOR: f32 = 0.3;
    const TILESET_DETAILS_HEIGHT_FACTOR: f32 = 0.5;

    pub fn new() -> Self {
        let left_toolbar = Toolbar::new(ToolbarPosition::Left, Self::LEFT_TOOLBAR_WIDTH);

        let right_toolbar = Toolbar::new(ToolbarPosition::Right, Self::RIGHT_TOOLBAR_WIDTH)
            .with_element(Self::LAYER_LIST_HEIGHT_FACTOR, LayerListElement::new())
            .with_element(Self::TILESET_LIST_HEIGHT_FACTOR, TilesetListElement::new())
            .with_element(Self::TILESET_DETAILS_HEIGHT_FACTOR, TilesetDetailsElement::new());

        EditorGui {
            left_toolbar,
            right_toolbar,
            open_windows: Vec::new(),
            context_menu: None,
        }
    }

    pub fn get_element_at(&self, position: Vec2) -> Option<GuiElement> {
        if let Some(context_menu) = &self.context_menu {
            if context_menu.contains(position) {
                return Some(GuiElement::ContextMenu);
            }
        }

        if self.left_toolbar.contains(position) || self.right_toolbar.contains(position) {
            return Some(GuiElement::Toolbar);
        }

        for window in &self.open_windows {
            if window.contains(position) {
                return Some(GuiElement::Window);
            }
        }

        None
    }

    pub fn is_element_at(&self, position: Vec2) -> bool {
        self.get_element_at(position).is_some()
    }

    pub fn open_context_menu(&mut self, position: Vec2) {
        let menu = ContextMenu::new(position, &[
            ContextMenuEntry::action("Undo", EditorAction::Undo),
            ContextMenuEntry::action("Redo", EditorAction::Redo),
        ]);

        self.context_menu = Some(menu);
    }

    pub fn close_context_menu(&mut self) {
        self.context_menu = None;
    }

    pub fn add_window(&mut self, window: Box<dyn Window>) {
        self.open_windows.push(window);
    }

    pub fn draw(&mut self, map: &Map, draw_params: EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let ui = &mut root_ui();

        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.editor_skins.default);
        }

        if let Some(action) = self.left_toolbar.draw(ui, map, &draw_params) {
            res = Some(action);
        }

        if let Some(action) = self.right_toolbar.draw(ui, map, &draw_params) {
            res = Some(action);
        }

        let mut i = 0;
        while i < self.open_windows.len() {
            let window = self.open_windows.get_mut(i).unwrap();
            let params = window.get_params().clone();

            let position = params.get_absolute_position();
            let size = params.size;

            let id = hash!("window", i);

            let mut should_close = false;
            widgets::Window::new(id, position, size).titlebar(false).movable(params.is_static == false).ui(ui, |ui| {
                let mut content_size = size - vec2(
                    EditorSkinCollection::WINDOW_MARGIN_LEFT + EditorSkinCollection::WINDOW_MARGIN_RIGHT,
                    EditorSkinCollection::WINDOW_MARGIN_TOP + EditorSkinCollection::WINDOW_MARGIN_BOTTOM,
                );

                let mut content_position = Vec2::ZERO;

                if let Some(title) = &params.title {
                    let gui_resources = storage::get::<GuiResources>();
                    ui.push_skin(&gui_resources.editor_skins.window_header);

                    ui.label(content_position, title);

                    let label_size = ui.calc_size(title);

                    content_size.y -= label_size.y;
                    content_position.y += label_size.y;

                    ui.pop_skin();
                }

                widgets::Group::new(hash!(id, "group"), content_size).position(content_position).ui(ui, |ui| {
                    if let Some(window_res) = window.draw(ui, size, map, &draw_params) {
                        if let WindowResult::Action(action) = window_res {
                            res = Some(action);
                        }

                        should_close = true;
                    }
                });
            });

            if should_close {
                self.open_windows.remove(i);
                continue;
            }

            i += 1;
        };

        if let Some(context_menu) = &mut self.context_menu {
            if let Some(action) = context_menu.draw(ui) {
                self.context_menu = None;
                res = Some(action);
            }
        }

        ui.pop_skin();

        res
    }
}
