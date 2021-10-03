pub mod windows;
pub mod toolbars;
pub mod context_menu;

pub mod skins;

use macroquad::{
    experimental::{
        collections::storage,
    },
    ui::{
        Id,
        Ui,
        widgets,
        root_ui,
    },
    hash,
    prelude::*,
};

use super::{
    EditorAction,
    UndoableAction,
};

use crate::{
    map::{
        Map,
        MapLayerKind,
        ObjectLayerKind,
    },
    gui::GuiResources,
};

use toolbars::{
    ToolbarPosition,
    Toolbar,
    ToolbarElement,
    ToolbarElementParams,
    create_left_toolbar,
    create_right_toolbar,
};

use windows::{
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
    create_layer_window: Option<CreateLayerWindow>,
    create_tileset_window: Option<CreateTilesetWindow>,
    context_menu: Option<ContextMenu>,
}

impl EditorGui {
    const LEFT_TOOLBAR_WIDTH: f32 = 50.0;
    const RIGHT_TOOLBAR_WIDTH: f32 = 250.0;

    pub fn new() -> Self {
        let left_toolbar = create_left_toolbar(Self::LEFT_TOOLBAR_WIDTH);
        let right_toolbar = create_right_toolbar(Self::RIGHT_TOOLBAR_WIDTH);

        EditorGui {
            left_toolbar,
            right_toolbar,
            create_layer_window: None,
            create_tileset_window: None,
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

        if let Some(window) = &self.create_layer_window {
            if window.contains(position) {
                return Some(GuiElement::Window);
            }
        }

        if let Some(window) = &self.create_tileset_window {
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

    pub fn open_create_layer_window(&mut self) {
        let window = CreateLayerWindow::new();
        self.create_layer_window = Some(window);
    }

    pub fn close_create_layer_window(&mut self) {
        self.create_layer_window = None;
    }

    pub fn open_create_tileset_window(&mut self) {
        let window = CreateTilesetWindow::new();
        self.create_tileset_window = Some(window);
    }

    pub fn close_create_tileset_window(&mut self) {
        self.create_tileset_window = None;
    }

    pub fn draw(&mut self, map: &Map, params: EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let gui_resources = storage::get::<GuiResources>();

        let ui = &mut root_ui();
        ui.push_skin(&gui_resources.editor_skins.default);

        res = self.left_toolbar.draw(ui, map, &params);
        res = self.right_toolbar.draw(ui, map, &params);

        if let Some(window) = &mut self.create_layer_window {
            res = window.draw(ui, map, &params);
            if res.is_some() {
                self.create_layer_window = None;
            }
        }

        if let Some(window) = &mut self.create_tileset_window {
            res = window.draw(ui, map, &params);
            if res.is_some() {
                self.create_tileset_window = None;
            }
        }

        if let Some(context_menu) = &mut self.context_menu {
            let was_some = res.is_some();
            res = context_menu.draw(ui);
            if was_some == false && res.is_some() {
                self.context_menu = None;
            }
        }

        ui.pop_skin();

        res
    }
}
