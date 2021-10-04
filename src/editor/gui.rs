use std::{
    any::{
        TypeId,
    },
    collections::HashMap,
};

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
    LayerListElement,
    TilesetListElement,
    TilesetDetailsElement,
};

use windows::{
    Window,
};

pub use windows::{
    ConfirmDialog,
    CreateLayerWindow,
    CreateTilesetWindow,
    TilesetPropertiesWindow,
};

use context_menu::{
    ContextMenu,
    ContextMenuEntry,
};

pub const NO_COLOR: Color = Color::new(0.0, 0.0, 0.0, 0.0);

pub const ELEMENT_MARGIN: f32 = 8.0;

pub const WINDOW_BUTTON_HEIGHT: f32 = 32.0;

pub const WINDOW_BUTTON_MIN_WIDTH: f32 = 64.0;
pub const WINDOW_BUTTON_MAX_WIDTH: f32 = 96.0;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GuiElement {
    ContextMenu,
    Toolbar,
    Window,
}

#[derive(Debug, Clone)]
pub struct ButtonParams {
    pub label: &'static str,
    pub width_override: Option<f32>,
    pub action: Option<EditorAction>,
}

impl Default for ButtonParams {
    fn default() -> Self {
        ButtonParams {
            label: "",
            width_override: None,
            action: None,
        }
    }
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
    open_windows: HashMap<TypeId, Box<dyn Window>>,
    context_menu: Option<ContextMenu>,
}

impl EditorGui {
    const LEFT_TOOLBAR_WIDTH: f32 = 50.0;
    const RIGHT_TOOLBAR_WIDTH: f32 = 250.0;

    const LAYER_LIST_HEIGHT_FACTOR: f32 = 0.3;
    const TILESET_LIST_HEIGHT_FACTOR: f32 = 0.2;
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
            open_windows: HashMap::new(),
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

        for (_, window) in &self.open_windows {
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

    pub fn add_window<W: Window + 'static>(&mut self, window: W) {
        let key = TypeId::of::<W>();
        if self.open_windows.contains_key(&key) == false {
            self.open_windows.insert(key, Box::new(window));
        }
    }

    pub fn remove_window<W: Window + 'static>(&mut self) {
        let key = TypeId::of::<W>();
        self.open_windows.remove(&key).unwrap();
    }

    pub fn remove_window_id(&mut self, id: TypeId) {
        self.open_windows.remove(&id).unwrap();
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

        for (id, window) in &mut self.open_windows {
            let params = window.get_params().clone();

            let position = params.get_absolute_position();
            let size = params.size;

            widgets::Window::new(hash!(id), position, size).titlebar(false).movable(params.is_static == false).ui(ui, |ui| {
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

                if params.has_buttons {
                    content_size.y -= WINDOW_BUTTON_HEIGHT + ELEMENT_MARGIN;
                }

                widgets::Group::new(hash!(id, "content"), content_size).position(content_position).ui(ui, |ui| {
                    if let Some(action) = window.draw(ui, content_size, map, &draw_params) {
                        res = Some(action);
                    }
                });

                if params.has_buttons {
                    let button_area_size = vec2(content_size.x, WINDOW_BUTTON_HEIGHT);
                    let button_area_position = vec2(content_position.x, content_size.y + ELEMENT_MARGIN);

                    widgets::Group::new(hash!(id, "buttons"), button_area_size).position(button_area_position).ui(ui, |ui| {
                        let mut button_position = Vec2::ZERO;

                        let buttons = window.get_buttons(map, &draw_params);

                        let button_cnt = buttons.len();
                        let margins = (button_cnt - 1) as f32 * ELEMENT_MARGIN;
                        let width = ((size.x - margins) / button_cnt as f32)
                            .clamp(WINDOW_BUTTON_MIN_WIDTH, WINDOW_BUTTON_MAX_WIDTH);

                        let button_size = vec2(width, WINDOW_BUTTON_HEIGHT);

                        for button in buttons {
                            if button.action.is_none() {
                                let gui_resources = storage::get::<GuiResources>();
                                ui.push_skin(&gui_resources.editor_skins.button_disabled);
                            }

                            let was_clicked = widgets::Button::new(button.label)
                                .position(button_position)
                                .size(button_size)
                                .ui(ui);


                            if button.action.is_some() {
                                if was_clicked {
                                    res = button.action;
                                }
                            } else {
                                ui.pop_skin();
                            }

                            button_position.x += button_size.x + ELEMENT_MARGIN;
                        }
                    });
                }
            });
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
