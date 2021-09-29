use std::ops::Deref;

use macroquad::{
    experimental::{
        collections::storage,
    },
    ui::{
        Ui,
        hash,
        widgets,
    },
    prelude::*,
};

use crate::{
    gui::GuiResources,
    map::{
        MapLayerKind,
        ObjectLayerKind,
    },
    editor::actions::EditorAction,
};

#[derive(Debug, Clone)]
pub enum ContextMenuEntry {
    Separator,
    Action {
        label: String,
        action: EditorAction,
    },
    SubMenu {
        label: String,
        entries: Vec<ContextMenuEntry>,
        is_open: bool,
    },
}

impl ContextMenuEntry {
    const WIDTH: f32 = 150.0;
    const ENTRY_HEIGHT: f32 = 25.0;
    const SEPARATOR_HEIGHT: f32 = 10.0;

    pub fn separator() -> Self {
        ContextMenuEntry::Separator
    }

    pub fn action(label: &str, action: EditorAction) -> Self {
        ContextMenuEntry::Action { label: label.to_string(), action }
    }

    pub fn sub_menu(label: &str, entries: &[ContextMenuEntry]) -> Self {
        ContextMenuEntry::SubMenu { label: label.to_string(), entries: entries.to_vec(), is_open: false }
    }

    fn get_height(&self) -> f32 {
        if let Self::Separator = self {
            return Self::SEPARATOR_HEIGHT;
        }

        Self::ENTRY_HEIGHT
    }

    fn contains(&self, position: Vec2, point: Vec2) -> bool {
        let rect = Rect::new(position.x, position.y, Self::WIDTH, self.get_height());
        if rect.contains(point) {
            return true;
        }

        if let Self::SubMenu { entries, is_open, .. } = self {
            if *is_open {
                let mut position = vec2(position.x + Self::WIDTH, position.y);
                position = get_corrected_context_menu_position(position, entries, false);

                for entry in entries {
                    if entry.contains(position, point) {
                        return true;
                    }

                    position.y += entry.get_height();
                }
            }
        }

        false
    }
}

pub struct ContextMenu {
    entries: Vec<ContextMenuEntry>,
    position: Vec2,
}

impl ContextMenu {
    pub fn new(position: Vec2, entries: &[ContextMenuEntry]) -> Self {
        let entries = entries.to_vec();
        let position = get_corrected_context_menu_position(position, &entries, true);

        ContextMenu {
            entries,
            position,
        }
    }

    pub fn get_size(&self) -> Vec2 {
        let mut height = 0.0;
        for entry in &self.entries {
            height += entry.get_height();
        }

        vec2(ContextMenuEntry::WIDTH, height)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        let mut position = self.position;
        for entry in &self.entries {
            if entry.contains(position, point) {
                return true;
            }

            position.y += entry.get_height();
        }

        false
    }

    pub fn draw(&mut self, ui: &mut Ui) -> Option<EditorAction> {
        let mut res = None;

        let gui_resources = storage::get::<GuiResources>();
        ui.push_skin(&gui_resources.skins.editor_context_menu_skin);

        res = draw_context_menu_entries(ui, self.position, &mut self.entries);

        ui.pop_skin();

        res
    }
}

fn get_corrected_context_menu_position(position: Vec2, entries: &[ContextMenuEntry], is_root: bool) -> Vec2 {
    let mut height = 0.0;

    for entry in entries {
        height += entry.get_height();
    }

    let screen_width = screen_width();
    let screen_height = screen_height();

    let x = if is_root {
        position.x.clamp(0.0, screen_width - ContextMenuEntry::WIDTH)
    } else if position.x + ContextMenuEntry::WIDTH > screen_width {
        position.x - ContextMenuEntry::WIDTH * 2.0
    } else {
        position.x
    };

    let y = position.y.clamp(0.0, screen_height - height);

    vec2(x, y)
}

fn draw_context_menu_entries(ui: &mut Ui, position: Vec2, entries: &mut [ContextMenuEntry]) -> Option<EditorAction> {
    let mut res = None;

    let mut size = vec2(ContextMenuEntry::WIDTH, 0.0);
    for entry in &mut *entries {
        size.y += entry.get_height();
    }

    let mut sub_menus = Vec::new();

    widgets::Group::new(hash!(), size)
        .position(position)
        .ui(ui, |ui| {
            let mut y_offset = 0.0;

            for entry in entries {
                let size = vec2(ContextMenuEntry::WIDTH, entry.get_height());
                let entry_position = vec2(0.0, y_offset);

                match entry {
                    ContextMenuEntry::Separator => {
                        widgets::Group::new(hash!(), size)
                            .position(entry_position)
                            .ui(ui, |ui| {
                                // TODO: draw separator
                            });
                    }
                    ContextMenuEntry::Action { label, action } => {
                        let button = widgets::Button::new("")
                            .position(entry_position)
                            .size(size)
                            .ui(ui);

                        ui.label(entry_position, label);

                        if button {
                            res = Some(action.clone());
                        }
                    }
                    ContextMenuEntry::SubMenu { label, entries, is_open } => {
                        let button = widgets::Button::new("")
                            .position(entry_position)
                            .size(size)
                            .ui(ui);

                        ui.label(entry_position, label);

                        {
                            let suffix = if *is_open { "<" } else { ">" };
                            let suffix_width = ui.calc_size(suffix).x;
                            let position = vec2(entry_position.x + size.x - suffix_width, entry_position.y);
                            ui.label(position, suffix);
                        }

                        if button {
                            *is_open = !*is_open;
                        }

                        if *is_open {
                            let position = position + vec2(entry_position.x + ContextMenuEntry::WIDTH, entry_position.y);
                            sub_menus.push((position, entries));
                        }
                    }
                }

                y_offset += size.y;
            }
        });

    for (position, entries) in sub_menus {
        let position = get_corrected_context_menu_position(position, entries, false);
        res = draw_context_menu_entries(ui, position, entries);
    }

    res
}

pub const RIGHT_MENUBAR_WIDTH: f32 = 150.0;
const MENU_ENTRY_HEIGHT: f32 = 25.0;
const LAYER_LIST_HEIGHT_FACTOR: f32 = 0.4;

pub fn draw_right_menu_bar(ui: &mut Ui, current_layer: Option<String>, layers: &[(String, MapLayerKind)]) -> Option<EditorAction> {
    let mut res = None;

    let screen_height = screen_height();
    let screen_width = screen_width();

    let size = vec2(RIGHT_MENUBAR_WIDTH, screen_height);
    let position = vec2(screen_width - size.x, 0.0);

    let gui_resources = storage::get::<GuiResources>();
    ui.push_skin(&gui_resources.skins.editor_menu_skin);

    widgets::Group::new(hash!(), size)
        .position(position)
        .ui(ui, |ui| {
            let mut position = Vec2::ZERO;
            let entry_size = vec2(RIGHT_MENUBAR_WIDTH, MENU_ENTRY_HEIGHT);

            ui.push_skin(&gui_resources.skins.editor_menu_bg);
            widgets::Button::new("").position(position).size(size).ui(ui);
            ui.pop_skin();

            ui.push_skin(&gui_resources.skins.editor_menu_header_bg);
            widgets::Button::new("").position(position).size(entry_size).ui(ui);
            ui.label(position, "Layers");
            ui.pop_skin();

            position.y += MENU_ENTRY_HEIGHT;

            let size = vec2(RIGHT_MENUBAR_WIDTH, (screen_height * LAYER_LIST_HEIGHT_FACTOR) - MENU_ENTRY_HEIGHT);

            widgets::Group::new(hash!(), size).position(position).ui(ui, |ui| {
                let mut position = Vec2::ZERO;

                for (id, kind) in layers {
                    let is_selected =
                        if let Some(current_layer) = &current_layer {
                            id == current_layer
                        } else {
                            false
                        };

                    if is_selected {
                        ui.push_skin(&gui_resources.skins.editor_menu_selected_skin);
                    }

                    let button = widgets::Button::new("")
                        .size(entry_size)
                        .position(position)
                        .ui(ui);

                    ui.label(position, id.deref());

                    {
                        let suffix =
                            match kind {
                                MapLayerKind::TileLayer => "T",
                                MapLayerKind::ObjectLayer(kind) => {
                                    match kind {
                                        ObjectLayerKind::None => "O",
                                        ObjectLayerKind::Items => "I",
                                        ObjectLayerKind::SpawnPoints => "S",
                                    }
                                }
                            };

                        let position = vec2(position.x + RIGHT_MENUBAR_WIDTH - ui.calc_size(suffix).x, position.y);
                        ui.label(position, suffix);
                    }

                    if button {
                        let action = EditorAction::SelectLayer(id.clone());
                        res = Some(action);
                    }

                    if is_selected {
                        ui.pop_skin();
                    }

                    position.y += MENU_ENTRY_HEIGHT;
                }
            });
        });

    ui.pop_skin();

    res
}