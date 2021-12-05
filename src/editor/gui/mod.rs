use std::{any::TypeId, collections::HashMap};

pub mod context_menu;
pub mod toolbars;
pub mod windows;

pub mod combobox;

mod editor_menu;

pub use combobox::{ComboBoxBuilder, ComboBoxValue};

pub use editor_menu::{
    close_editor_menu, draw_editor_menu, is_editor_menu_open, open_editor_menu, toggle_editor_menu,
    EDITOR_MENU_RESULT_MAIN_MENU, EDITOR_MENU_RESULT_NEW, EDITOR_MENU_RESULT_OPEN,
    EDITOR_MENU_RESULT_QUIT, EDITOR_MENU_RESULT_SAVE, EDITOR_MENU_RESULT_SAVE_AS,
};

use macroquad::{
    experimental::collections::storage,
    hash,
    prelude::*,
    ui::{root_ui, widgets},
};

use super::{EditorAction, EditorContext};

use crate::{
    gui::{GuiResources, ELEMENT_MARGIN},
    map::Map,
};

pub use toolbars::{
    LayerListElement, ObjectListElement, TilesetDetailsElement, TilesetListElement,
    ToolSelectorElement, Toolbar, ToolbarElement, ToolbarElementParams, ToolbarPosition,
};

pub use windows::{
    ConfirmDialog, CreateLayerWindow, CreateObjectWindow, CreateTilesetWindow,
    TilesetPropertiesWindow, Window, WINDOW_BUTTON_MAX_WIDTH, WINDOW_BUTTON_MIN_WIDTH,
};

use crate::gui::{BUTTON_FONT_SIZE, BUTTON_MARGIN_V, WINDOW_MARGIN_H, WINDOW_MARGIN_V};
use crate::map::MapLayerKind;
use context_menu::{ContextMenu, ContextMenuEntry};

#[derive(Debug, Default, Clone)]
pub struct ButtonParams {
    pub label: &'static str,
    // This should be an absolute width for window and a width factor for toolbar elements.
    // Permitted width factors for toolbar element buttons are 0.25 and 0.5.
    pub width_override: Option<f32>,
    // This holds the action that will be applied on click.
    // Setting this to `None` will disable the button.
    pub action: Option<EditorAction>,
}

pub struct EditorGui {
    left_toolbar: Option<Toolbar>,
    right_toolbar: Option<Toolbar>,
    open_windows: HashMap<TypeId, Box<dyn Window>>,
    context_menu: Option<ContextMenu>,
}

impl EditorGui {
    pub const LEFT_TOOLBAR_WIDTH: f32 = 82.0;
    pub const RIGHT_TOOLBAR_WIDTH: f32 = 250.0;

    pub const TOOL_SELECTOR_HEIGHT_FACTOR: f32 = 0.5;
    pub const LAYER_LIST_HEIGHT_FACTOR: f32 = 0.3;
    pub const TILESET_LIST_HEIGHT_FACTOR: f32 = 0.2;
    pub const TILESET_DETAILS_HEIGHT_FACTOR: f32 = 0.5;
    pub const OBJECT_LIST_HEIGHT_FACTOR: f32 = 0.7;

    pub fn new() -> Self {
        EditorGui {
            left_toolbar: None,
            right_toolbar: None,
            open_windows: HashMap::new(),
            context_menu: None,
        }
    }

    pub fn add_toolbar(&mut self, toolbar: Toolbar) {
        match toolbar.position {
            ToolbarPosition::Left => {
                self.left_toolbar = Some(toolbar);
            }
            ToolbarPosition::Right => {
                self.right_toolbar = Some(toolbar);
            }
        }
    }

    pub fn with_toolbar(self, toolbar: Toolbar) -> Self {
        let mut gui = self;
        gui.add_toolbar(toolbar);
        gui
    }

    pub fn context_menu_contains(&self, position: Vec2) -> bool {
        if let Some(context_menu) = &self.context_menu {
            if context_menu.contains(position) {
                return true;
            }
        }

        false
    }

    pub fn contains(&self, position: Vec2) -> bool {
        if self.context_menu_contains(position) {
            return true;
        }

        if let Some(left_toolbar) = &self.left_toolbar {
            if left_toolbar.contains(position) {
                return true;
            }
        }

        if let Some(right_toolbar) = &self.right_toolbar {
            if right_toolbar.contains(position) {
                return true;
            }
        }

        for window in self.open_windows.values() {
            if window.contains(position) {
                return true;
            }
        }

        false
    }

    pub fn open_context_menu(&mut self, position: Vec2, map: &Map, ctx: EditorContext) {
        let mut entries = vec![
            ContextMenuEntry::action("Undo", EditorAction::Undo),
            ContextMenuEntry::action("Redo", EditorAction::Redo),
        ];

        if let Some(layer_id) = &ctx.selected_layer {
            let layer = &map.layers.get(layer_id).unwrap();
            if layer.kind == MapLayerKind::ObjectLayer {
                entries.push(ContextMenuEntry::action(
                    "Create Object",
                    EditorAction::OpenCreateObjectWindow {
                        position,
                        layer_id: layer_id.clone(),
                    },
                ));
            }
        }

        entries.append(&mut vec![
            ContextMenuEntry::action("Add Layer", EditorAction::OpenCreateLayerWindow),
            ContextMenuEntry::action("Background", EditorAction::OpenBackgroundPropertiesWindow),
        ]);

        self.context_menu = Some(ContextMenu::new(position, &entries));
    }

    pub fn close_context_menu(&mut self) {
        self.context_menu = None;
    }

    pub fn add_window<W: Window + 'static>(&mut self, window: W) {
        let key = TypeId::of::<W>();
        self.open_windows
            .entry(key)
            .or_insert_with(|| Box::new(window));
    }

    pub fn remove_window<W: Window + 'static>(&mut self) {
        let key = TypeId::of::<W>();
        self.open_windows.remove(&key).unwrap();
    }

    pub fn remove_window_id(&mut self, id: TypeId) {
        self.open_windows.remove(&id).unwrap();
    }

    pub fn draw(&mut self, map: &Map, ctx: EditorContext) -> Option<EditorAction> {
        let mut res = None;

        let ui = &mut root_ui();

        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.skins.default);
        }

        if let Some(left_toolbar) = &mut self.left_toolbar {
            if let Some(action) = left_toolbar.draw(ui, map, &ctx) {
                res = Some(action);
            }
        }

        if let Some(right_toolbar) = &mut self.right_toolbar {
            if let Some(action) = right_toolbar.draw(ui, map, &ctx) {
                res = Some(action);
            }
        }

        for (id, window) in &mut self.open_windows {
            let params = window.get_params().clone();

            let position = params.get_absolute_position();
            let size = params.size;

            widgets::Window::new(hash!(id), position, size)
                .titlebar(false)
                .movable(!params.is_static)
                .ui(ui, |ui| {
                    let mut content_size =
                        size - vec2(WINDOW_MARGIN_H * 2.0, WINDOW_MARGIN_V * 2.0);

                    let mut content_position = Vec2::ZERO;

                    if let Some(title) = &params.title {
                        let gui_resources = storage::get::<GuiResources>();
                        ui.push_skin(&gui_resources.skins.window_header);

                        ui.label(content_position, title);

                        let label_size = ui.calc_size(title);

                        content_size.y -= label_size.y;
                        content_position.y += label_size.y;

                        ui.pop_skin();
                    }

                    let button_height = BUTTON_FONT_SIZE + (BUTTON_MARGIN_V * 2.0);

                    if params.has_buttons {
                        content_size.y -= button_height + ELEMENT_MARGIN;
                    }

                    widgets::Group::new(hash!(id, "content"), content_size)
                        .position(content_position)
                        .ui(ui, |ui| {
                            if let Some(action) = window.draw(ui, content_size, map, &ctx) {
                                res = Some(action);
                            }
                        });

                    if params.has_buttons {
                        let button_area_size = vec2(content_size.x, button_height);
                        let button_area_position =
                            vec2(content_position.x, content_position.y + content_size.y);

                        widgets::Group::new(hash!(id, "buttons"), button_area_size)
                            .position(button_area_position)
                            .ui(ui, |ui| {
                                let mut button_position = vec2(ELEMENT_MARGIN, 0.0);

                                let buttons = window.get_buttons(map, &ctx);

                                let button_cnt = buttons.len();
                                let margins = button_cnt as f32 * ELEMENT_MARGIN;
                                let width = ((size.x - margins) / button_cnt as f32)
                                    .clamp(WINDOW_BUTTON_MIN_WIDTH, WINDOW_BUTTON_MAX_WIDTH);

                                let button_size = vec2(width, button_height);

                                for button in buttons {
                                    if button.action.is_none() {
                                        let gui_resources = storage::get::<GuiResources>();
                                        ui.push_skin(&gui_resources.skins.button_disabled);
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
        }

        if let Some(context_menu) = &mut self.context_menu {
            if let Some(action) = context_menu.draw(ui) {
                self.context_menu = None;
                res = Some(action);
            }
        }

        if is_editor_menu_open() {
            if let Some(menu_res) = draw_editor_menu(ui, &ctx) {
                close_editor_menu();

                match menu_res.into_usize() {
                    EDITOR_MENU_RESULT_NEW => {
                        let action = EditorAction::OpenCreateMapWindow;
                        res = Some(action);
                    }
                    EDITOR_MENU_RESULT_OPEN => {
                        let action = EditorAction::OpenLoadMapWindow;
                        res = Some(action);
                    }
                    EDITOR_MENU_RESULT_SAVE => {
                        let action = EditorAction::SaveMap(None);
                        res = Some(action);
                    }
                    EDITOR_MENU_RESULT_SAVE_AS => {
                        let action = EditorAction::OpenSaveMapWindow;
                        res = Some(action);
                    }
                    EDITOR_MENU_RESULT_MAIN_MENU => {
                        let action = EditorAction::ExitToMainMenu;
                        res = Some(action);
                    }
                    EDITOR_MENU_RESULT_QUIT => {
                        let action = EditorAction::QuitToDesktop;
                        res = Some(action);
                    }
                    _ => {}
                }
            }
        }

        ui.pop_skin();

        res
    }
}

impl Default for EditorGui {
    fn default() -> Self {
        Self::new()
    }
}
