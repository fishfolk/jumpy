pub mod menus;
pub mod window_builder;

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
        MapLayerKind,
        ObjectLayerKind,
    },
    gui::GuiResources,
};

use menus::{
    ContextMenu,
    ContextMenuEntry,
    CreateLayerWindow,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EditorGuiElement {
    None,
    ContextMenu,
    LayerList,
    RightMenuBar,
    LayerListEntry(usize),
}

pub struct EditorGui {
    pub context_menu: Option<ContextMenu>,
    pub create_layer_menu: Option<CreateLayerWindow>,
}

impl EditorGui {
    pub fn new() -> Self {
        EditorGui {
            context_menu: None,
            create_layer_menu: None,
        }
    }

    pub fn draw(&mut self, current_layer: Option<String>, layers: &[(String, MapLayerKind)]) -> Option<EditorAction> {
        let gui_resources = storage::get::<GuiResources>();

        let mut res = None;

        let ui = &mut root_ui();
        ui.push_skin(&gui_resources.skins.editor_skin);

        res = menus::draw_right_menu_bar(ui, current_layer, layers);

        if let Some(create_layer_menu) = &mut self.create_layer_menu {
            let was_some = res.is_some();
            res = create_layer_menu.draw(ui, layers);
            if was_some == false && res.is_some() {
                self.create_layer_menu = None;
            }
        }

        if let Some(context_menu) = &mut self.context_menu {
            let was_some = res.is_some();
             res = context_menu.draw(ui);
            if was_some == false && res.is_some() {
                self.context_menu = None;
            }
        }

        res
    }

    pub fn open_context_menu(&mut self, position: Vec2, entries: &[ContextMenuEntry]) {
        if entries.len() > 0 {
            let menu = ContextMenu::new(position, entries);
            self.context_menu = Some(menu);
        }
    }

    pub fn close_context_menu(&mut self) {
        self.context_menu = None;
    }

    pub fn open_create_layer_menu(&mut self, index: usize) {
        let menu = CreateLayerWindow::new(index);
        self.create_layer_menu = Some(menu);
    }

    pub fn close_create_layer_menu(&mut self) {
        self.create_layer_menu = None;
    }
}
