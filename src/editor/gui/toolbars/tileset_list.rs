use core::prelude::*;

use super::{
    EditorAction, EditorContext, GuiResources, Map, Toolbar, ToolbarElement, ToolbarElementParams,
};

use crate::{editor::gui::ButtonParams, map::MapLayerKind};
use crate::macroquad::ui::{Ui, widgets};

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

        TilesetListElement { params }
    }
}

impl ToolbarElement for TilesetListElement {
    fn get_params(&self) -> &ToolbarElementParams {
        &self.params
    }

    fn get_buttons(&self, _map: &Map, ctx: &EditorContext) -> Vec<ButtonParams> {
        // Properties are disabled until autotile is up and running

        let mut delete_action = None;
        //let mut properties_action = None;

        if let Some(tileset_id) = &ctx.selected_tileset {
            delete_action = Some(EditorAction::DeleteTileset(tileset_id.clone()));
            /*
            properties_action = Some(EditorAction::OpenTilesetPropertiesWindow(
                tileset_id.clone(),
            ));
            */
        }

        vec![
            ButtonParams {
                label: "+",
                width_override: Some(0.25),
                action: Some(EditorAction::OpenCreateTilesetWindow),
            },
            ButtonParams {
                label: "-",
                width_override: Some(0.25),
                action: delete_action,
            },
            ButtonParams {
                label: "Edit",
                width_override: Some(0.5),
                //action: properties_action,
                action: None,
            },
        ]
    }

    fn draw(
        &mut self,
        ui: &mut Ui,
        size: Vec2,
        map: &Map,
        ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let mut res = None;

        let entry_size = vec2(size.x, Toolbar::LIST_ENTRY_HEIGHT);
        let mut position = Vec2::ZERO;

        let gui_resources = storage::get::<GuiResources>();
        ui.push_skin(&gui_resources.skins.list_box);

        for tileset_id in map.tilesets.keys() {
            let is_selected = if let Some(selected_id) = &ctx.selected_tileset {
                tileset_id == selected_id
            } else {
                false
            };

            if is_selected {
                ui.push_skin(&gui_resources.skins.list_box_selected);
            }

            let was_clicked = widgets::Button::new("")
                .size(entry_size)
                .position(position)
                .ui(ui);

            ui.label(position, tileset_id);

            if was_clicked {
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

    fn is_drawn(&self, map: &Map, ctx: &EditorContext) -> bool {
        if let Some(layer_id) = &ctx.selected_layer {
            if let Some(layer) = map.layers.get(layer_id) {
                return layer.kind == MapLayerKind::TileLayer;
            }
        }

        false
    }
}

impl Default for TilesetListElement {
    fn default() -> Self {
        Self::new()
    }
}
