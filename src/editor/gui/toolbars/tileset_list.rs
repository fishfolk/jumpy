use macroquad::{
    ui::{
        Id,
        Ui,
        hash,
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

pub struct TilesetList {
    params: ToolbarElementParams,
}

impl TilesetList {
    pub fn new() -> Box<Self> {
        let params = ToolbarElementParams {
            id: hash!("tileset_list"),
            header: Some("Tilesets".to_string()),
            has_menubar: true,
            has_margins: false,
        };

        Box::new(TilesetList {
            params,
        })
    }
}

impl ToolbarElement for TilesetList {
    fn get_params(&self) -> ToolbarElementParams {
        self.params.clone()
    }

    fn draw(&mut self, ui: &mut Ui, size: Vec2, map: &Map, draw_params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let entry_size = vec2(size.x, Toolbar::LIST_ENTRY_HEIGHT);
        let mut position = Vec2::ZERO;

        let gui_resources = storage::get::<GuiResources>();
        ui.push_skin(&gui_resources.editor_skins.menu);

        for (id, _) in &map.tilesets {
            let is_selected = if let Some(selected_id) = &draw_params.selected_tileset {
                id == selected_id
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

            ui.label(position, id);

            if button {
                res = Some(EditorAction::SelectTileset(id.clone()));
            }

            if is_selected {
                ui.pop_skin();
            }

            position.y += entry_size.y;
        }

        ui.pop_skin();

        res
    }

    fn draw_menubar(&mut self, ui: &mut Ui, size: Vec2, _map: &Map, draw_params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let mut position = Vec2::ZERO;

        let button_size = vec2(size.x * 0.25, size.y);

        let create_btn = widgets::Button::new("+")
            .size(button_size)
            .position(position)
            .ui(ui);

        if create_btn {
            res = Some(EditorAction::OpenCreateTilesetWindow);
        }

        position.x += button_size.x;

        let delete_btn = widgets::Button::new("-")
            .size(button_size)
            .position(position)
            .ui(ui);

        if delete_btn {
            if let Some(id) = draw_params.selected_tileset.clone() {
                res = Some(EditorAction::DeleteTileset(id));
            }
        }

        res
    }
}