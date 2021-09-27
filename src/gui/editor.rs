use std::ops::Deref;

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

use crate::{
    nodes::editor::{
        EditorAction,
    },
    gui::GuiResources,
};

#[derive(Debug, Clone)]
enum ContextMenuEntryKind {
    Action(EditorAction),
    SubMenu {
        entries: Vec<ContextMenuEntry>,
        is_open: bool,
    },
    Separator,
}

#[derive(Debug, Clone)]
pub struct ContextMenuEntry {
    pub title: Option<String>,
    kind: ContextMenuEntryKind,
}

impl ContextMenuEntry {
    fn new(title: Option<String>, kind: ContextMenuEntryKind) -> Self {
        ContextMenuEntry {
            title,
            kind,
        }
    }

    pub fn new_action(title: &str, action: EditorAction) -> Self {
        let title = Some(title.to_string());
        let kind = ContextMenuEntryKind::Action(action);

        Self::new(title, kind)
    }

    pub fn new_sub_menu(title: &str, entries: &[ContextMenuEntry]) -> Self {
        let title = Some(title.to_string());
        let kind = ContextMenuEntryKind::SubMenu {
            entries: entries.to_vec(),
            is_open: false,
        };

        Self::new(title, kind)
    }

    pub fn new_separator(title: Option<String>) -> Self {
        let kind = ContextMenuEntryKind::Separator;
        Self::new(title, kind)
    }
}

#[derive(Debug, Copy, Clone)]
enum ContextMenuResult {
    None,
    ToggleSubMenu,
    Action(EditorAction),
}

struct ContextMenu {
    pub position: Vec2,
    pub entries: Vec<ContextMenuEntry>,
    pub selection: usize,
}

impl ContextMenu {
    const WIDTH: f32 = 100.0;
    const ENTRY_HEIGHT: f32 = 25.0;
    const MAX_HEIGHT: f32 = 150.0;
    const SCROLLBAR_WIDTH: f32 = 10.0;

    fn new(position: Vec2, entries: &[ContextMenuEntry]) -> Self {
        ContextMenu {
            position,
            entries: entries.to_vec(),
            selection: 0,
        }
    }

    fn draw(&mut self, ui: &mut Ui) -> Option<EditorAction> {
        let gui_resources = storage::get::<GuiResources>();

        ui.push_skin(&gui_resources.skins.editor_context_menu_skin);
        let res = draw_context_menu_entries(ui, hash!(), self.position, &mut self.entries);
        ui.pop_skin();

        res
    }

    fn is_mouse_over(&self) -> bool {
        let mouse_position = {
            let (x, y) = mouse_position();
            vec2(x, y)
        };

        is_mouse_over_entries(mouse_position, self.position, &self.entries)
    }
}

fn is_mouse_over_entries(mouse_position: Vec2, position: Vec2, entries: &[ContextMenuEntry]) -> bool {
    let size = vec2(ContextMenu::WIDTH, ContextMenu::ENTRY_HEIGHT * entries.len() as f32);

    let screen_height = screen_height();
    let mut position = position;
    if position.y + size.y > screen_height {
        position.y = screen_height - size.y;
    }

    let rect = Rect::new(position.x, position.y, size.x, size.y);
    if rect.contains(mouse_position) {
        return true;
    }

    let mut i = 0;
    for entry in entries {
        if let ContextMenuEntryKind::SubMenu { entries, is_open } = &entry.kind {
            let position = vec2(position.x + ContextMenu::WIDTH, position.y + (i as f32 * ContextMenu::ENTRY_HEIGHT));
            if *is_open && is_mouse_over_entries(mouse_position, position, entries) {
                return true;
            }
        }

        i += 1;
    }

    false
}

fn draw_context_menu_entries(ui: &mut Ui, id: Id, position: Vec2, entries: &mut [ContextMenuEntry]) -> Option<EditorAction> {
    use ContextMenuEntryKind::*;

    let mut res = None;

    let size = vec2(ContextMenu::WIDTH, ContextMenu::ENTRY_HEIGHT * entries.len() as f32);

    let screen_height = screen_height();
    let mut position = position;
    if position.y + size.y > screen_height {
        position.y = screen_height - size.y;
    }

    let mut sub_menus = Vec::new();

    widgets::Group::new(id, size).position(position).ui(ui, |ui| {
        let mut next_entry_y = 0.0;
        for entry in entries {
            let entry_position = vec2(0.0, next_entry_y);
            next_entry_y += ContextMenu::ENTRY_HEIGHT;

            if let Separator = entry.kind {
                let title = entry.title
                    .clone()
                    .unwrap_or("".to_string());

                ui.label(entry_position, title.deref());
            } else {
                let button = widgets::Button::new("")
                    .position(entry_position)
                    .size(vec2(ContextMenu::WIDTH, ContextMenu::ENTRY_HEIGHT))
                    .ui(ui);

                if let Some(title) = &entry.title {
                    ui.label(entry_position, title.deref());
                }

                if let SubMenu { entries: _, is_open } = entry.kind {
                    let chevron = if is_open { "<" } else { ">" };

                    let mut position = entry_position;
                    position.x += size.x - ui.calc_size(chevron).x;

                    ui.label(position, chevron);
                }

                if button {
                    match &mut entry.kind {
                        Action(action) => {
                            res = Some(*action);
                        }
                        SubMenu { entries: _, is_open } => {
                            *is_open = !*is_open;
                        }
                        _ => {}
                    }
                }

                if let SubMenu { entries: _, is_open } = entry.kind {
                    if entry_position.y < size.y && is_open {
                        let sub_menu = (
                            vec2(position.x + size.x, position.y + entry_position.y),
                            &mut entry.kind,
                        );

                        sub_menus.push(sub_menu);
                    }
                }
            }
        }
    });

    let mut i = 0;
    for (position, kind) in sub_menus {
        if let SubMenu { entries, is_open } = kind {
            res = draw_context_menu_entries(ui, hash!(id, "sub_menu", i), position, entries);
        }

        i += 1;
    }

    res
}

pub struct EditorGui {
    pub action: Option<EditorAction>,
    context_menu: Option<ContextMenu>,
}

impl EditorGui {
    pub fn new() -> Self {
        EditorGui {
            action: None,
            context_menu: None,
        }
    }

    pub fn draw(&mut self) -> Option<EditorAction> {
        let gui_resources = storage::get::<GuiResources>();

        let mut res = None;

        let ui = &mut root_ui();

        ui.push_skin(&gui_resources.skins.editor_skin);
        if let Some(context_menu) = self.context_menu.as_mut() {
            res = context_menu.draw(ui)
        };
        if res.is_some() {
            self.context_menu = None;
        }
        ui.pop_skin();

        res
    }

    pub fn show_context_menu(&mut self, position: Vec2, entries: &[ContextMenuEntry]) {
        if entries.len() > 0 {
            let menu = ContextMenu {
                position,
                entries: entries.to_vec(),
                selection: 0,
            };

            self.context_menu = Some(menu);
        }
    }

    pub fn hide_context_menu(&mut self) {
        self.context_menu = None;
    }

    pub fn is_mouse_over_context_menu(&self) -> bool {
        if let Some(context_menu) = &self.context_menu {
            return context_menu.is_mouse_over();
        }

        false
    }

    pub fn is_context_menu_open(&self) -> bool {
        self.context_menu.is_some()
    }
}