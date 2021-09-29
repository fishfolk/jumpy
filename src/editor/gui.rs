pub mod menus;

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
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EditorGuiElement {
    None,
    ContextMenu,
    LayerList,
    LayerListEntry(String),
}

pub struct EditorGui {
    context_menu: Option<ContextMenu>,
}

impl EditorGui {
    pub fn new() -> Self {
        EditorGui {
            context_menu: None,
        }
    }

    pub fn draw(&mut self, current_layer: Option<String>, layers: &[(String, MapLayerKind)]) -> Option<EditorAction> {
        let gui_resources = storage::get::<GuiResources>();

        let mut res = None;

        let ui = &mut root_ui();
        ui.push_skin(&gui_resources.skins.editor_skin);

        res = menus::draw_right_menu_bar(ui, current_layer, layers);

        if let Some(context_menu) = &mut self.context_menu {
            res = context_menu.draw(ui);
        };

        if res.is_some() {
            self.context_menu = None;
        }

        res
    }

    pub fn get_element_at(&self, point: Vec2) -> EditorGuiElement {
        if let Some(context_menu) = &self.context_menu {
            if context_menu.contains(point) {
                return EditorGuiElement::ContextMenu;
            }
        }

        if point.x >= screen_width() - menus::RIGHT_MENUBAR_WIDTH {
            return EditorGuiElement::LayerList;
        }

        EditorGuiElement::None
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
}
