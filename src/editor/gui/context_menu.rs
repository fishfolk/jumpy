use core::prelude::*;

use super::EditorAction;

use core::gui::ELEMENT_MARGIN;
use crate::GuiTheme;

use crate::macroquad::hash;
use crate::macroquad::ui::{Ui, widgets};

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

    pub fn separator() -> Self {
        ContextMenuEntry::Separator
    }

    pub fn action(label: &str, action: EditorAction) -> Self {
        ContextMenuEntry::Action {
            label: label.to_string(),
            action,
        }
    }

    pub fn sub_menu(label: &str, entries: &[ContextMenuEntry]) -> Self {
        ContextMenuEntry::SubMenu {
            label: label.to_string(),
            entries: entries.to_vec(),
            is_open: false,
        }
    }

    fn get_height(&self) -> f32 {
        if let Self::Separator = self {
            return ELEMENT_MARGIN;
        }

        Self::ENTRY_HEIGHT
    }

    fn contains(&self, position: Vec2, point: Vec2) -> bool {
        let rect = Rect::new(position.x, position.y, Self::WIDTH, self.get_height());
        if rect.contains(point) {
            return true;
        }

        if let Self::SubMenu {
            entries, is_open, ..
        } = self
        {
            if *is_open {
                let mut position = vec2(position.x + Self::WIDTH, position.y);
                position = get_corrected_position(position, entries, false);

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
        let position = get_corrected_position(position, &entries, true);

        ContextMenu { entries, position }
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
        let gui_theme = storage::get::<GuiTheme>();
        ui.push_skin(&gui_theme.context_menu);

        let res = draw_entries(ui, self.position, &mut self.entries);

        ui.pop_skin();

        res
    }
}

fn get_corrected_position(position: Vec2, entries: &[ContextMenuEntry], is_root: bool) -> Vec2 {
    let mut height = 0.0;

    for entry in entries {
        height += entry.get_height();
    }

    let viewport = get_viewport();

    let x = if is_root {
        position
            .x
            .clamp(0.0, viewport.width - ContextMenuEntry::WIDTH)
    } else if position.x + ContextMenuEntry::WIDTH > viewport.width {
        position.x - ContextMenuEntry::WIDTH * 2.0
    } else {
        position.x
    };

    let y = position.y.clamp(0.0, viewport.height - height);

    vec2(x, y)
}

fn draw_entries(
    ui: &mut Ui,
    position: Vec2,
    entries: &mut [ContextMenuEntry],
) -> Option<EditorAction> {
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
                            .ui(ui, |_ui| {
                                // TODO: Draw texture
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
                    ContextMenuEntry::SubMenu {
                        label,
                        entries,
                        is_open,
                    } => {
                        let button = widgets::Button::new("")
                            .position(entry_position)
                            .size(size)
                            .ui(ui);

                        ui.label(entry_position, label);

                        {
                            let suffix = if *is_open { "<" } else { ">" };
                            let suffix_width = ui.calc_size(suffix).x;
                            let position =
                                vec2(entry_position.x + size.x - suffix_width, entry_position.y);
                            ui.label(position, suffix);
                        }

                        if button {
                            *is_open = !*is_open;
                        }

                        if *is_open {
                            let position = position
                                + vec2(
                                    entry_position.x + ContextMenuEntry::WIDTH,
                                    entry_position.y,
                                );
                            sub_menus.push((position, entries));
                        }
                    }
                }

                y_offset += size.y;
            }
        });

    for (position, entries) in sub_menus {
        let position = get_corrected_position(position, entries, false);
        res = draw_entries(ui, position, entries);
    }

    res
}
