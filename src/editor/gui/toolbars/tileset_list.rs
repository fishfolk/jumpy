use macroquad::{
    ui::{
        Ui,
        widgets,
    },
    experimental::{
        collections::storage,
    },
    prelude::*,
};

use super::{
    GuiResources,
    Toolbar,
    ToolbarElement,
    ToolbarElementParams,
    EditorDrawParams,
    EditorAction,
    Map,
};
use crate::editor::gui::ButtonParams;

pub struct TilesetListElement {
    params: ToolbarElementParams,
}

impl TilesetListElement {
    pub fn new() -> Self {
        let params = ToolbarElementParams {
            header: Some("Tilesets".to_string()),
            has_buttons: true,
            has_margins: false,
        };

        TilesetListElement {
            params,
        }
    }
}

impl ToolbarElement for TilesetListElement {
    fn get_params(&self) -> &ToolbarElementParams {
        &self.params
    }

    fn get_buttons(&self, _map: &Map, draw_params: &EditorDrawParams) -> Vec<ButtonParams> {
        let mut res = Vec::new();

        let mut action = None;
        if let Some(tileset_id) = draw_params.selected_tileset.clone() {
            action = Some(EditorAction::DeleteTileset(tileset_id));
        }

        res.push(ButtonParams {
            label: "+",
            width_override: Some(0.25),
            action: Some(EditorAction::OpenCreateTilesetWindow),
            ..Default::default()
        });

        res.push(ButtonParams {
            label: "-",
            width_override: Some(0.25),
            action,
            ..Default::default()
        });

        res
    }

    fn draw(&mut self, ui: &mut Ui, size: Vec2, map: &Map, draw_params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let entry_size = vec2(size.x, Toolbar::LIST_ENTRY_HEIGHT);
        let mut position = Vec2::ZERO;

        let gui_resources = storage::get::<GuiResources>();
        ui.push_skin(&gui_resources.editor_skins.menu);

        for (tileset_id, _) in &map.tilesets {
            let is_selected = if let Some(selected_id) = &draw_params.selected_tileset {
                tileset_id == selected_id
            } else {
                false
            };

            if is_selected {
                ui.push_skin(&gui_resources.editor_skins.menu_selected);
            }

            let button = widgets::Button::new("")
                .size(entry_size)
                .position(position)
                .ui(ui);

            ui.label(position, tileset_id);

            if button {
                res = Some(EditorAction::SelectTileset(tileset_id.clone()));
            }

            if is_selected {
                ui.pop_skin();
            }

            position.y += entry_size.y;
        }

        ui.pop_skin();

        res
    }
}