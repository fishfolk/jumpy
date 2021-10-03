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

use crate::{
    Resources,
};

use super::{
    Map,
    GuiResources,
    Toolbar,
    ToolbarElementParams,
    ToolbarElement,
    EditorAction,
    EditorDrawParams,
};

pub struct TilesetDetailsElement {
    params: ToolbarElementParams,
}

impl TilesetDetailsElement {
    pub fn new() -> Box<Self> {
        let params = ToolbarElementParams {
            header: None,
            has_menubar: true,
            has_margins: true,
        };

        Box::new(TilesetDetailsElement {
            params,
        })
    }
}

impl ToolbarElement for TilesetDetailsElement {
    fn get_params(&self) -> ToolbarElementParams {
        self.params.clone()
    }

    fn draw(&mut self, ui: &mut Ui, size: Vec2, map: &Map, draw_params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let mut position = Vec2::ZERO;

        if let Some(tileset_id) = &draw_params.selected_tileset {
            let tileset = map.tilesets.get(tileset_id).unwrap();

            let texture = {
                let resources = storage::get::<Resources>();
                resources.textures.get(&tileset.texture_id).cloned().unwrap()
            };

            let grid_size = vec2(tileset.grid_size.x as f32, tileset.grid_size.y as f32);

            let scaled_width = size.x;
            let scaled_height = (scaled_width / texture.width()) * texture.height();

            let scaled_tile_size = vec2(scaled_width / grid_size.x, scaled_height / grid_size.y);

            widgets::Texture::new(texture)
                .position(position)
                .size(scaled_width, scaled_height)
                .ui(ui);

            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.editor_skins.toolbar_tileset_grid);

            for y in 0..tileset.grid_size.y {
                for x in 0..tileset.grid_size.x {
                    let tile_id = y * tileset.grid_size.x + x;

                    let is_selected = if let Some(selected) = draw_params.selected_tile {
                        selected == tile_id
                    } else {
                        false
                    };

                    if is_selected {
                        ui.push_skin(&gui_resources.editor_skins.toolbar_tileset_grid_selected);
                    }

                    let position = vec2(x as f32, y as f32) * scaled_tile_size;

                    let button = widgets::Button::new("")
                        .size(scaled_tile_size)
                        .position(position)
                        .ui(ui);

                    if is_selected {
                        ui.pop_skin();
                    }

                    if button {
                        res = Some(EditorAction::SelectTile {
                            id: tile_id,
                            tileset_id: tileset.id.clone(),
                        });
                    }
                }
            }

            ui.pop_skin();

            position.y += scaled_height + Toolbar::MARGIN;
        }

        res
    }

    fn draw_menubar(&mut self, ui: &mut Ui, size: Vec2, _map: &Map, draw_params: &EditorDrawParams) -> Option<EditorAction> {
        let mut res = None;

        let mut position = Vec2::ZERO;

        {
            let button_size = vec2(size.x * 0.25, size.y);

            let zoom_in_btn = widgets::Button::new("+")
                .size(button_size)
                .position(position)
                .ui(ui);

            if zoom_in_btn {
                if let Some(tileset_id) = draw_params.selected_tileset.clone() {
                    //res = Some(EditorAction::OpenTileAttributesWindow(tileset_id));
                }
            }

            position.x += button_size.x;

            let zoom_out_btn = widgets::Button::new("-")
                .size(button_size)
                .position(position)
                .ui(ui);

            if zoom_out_btn {
                if let Some(tileset_id) = draw_params.selected_tileset.clone() {
                    //res = Some(EditorAction::OpenTileAttributesWindow(tileset_id));
                }
            }

            position.x += button_size.x;
        }

        let button_size = vec2(size.x * 0.5, size.y);

        let advanced_btn = widgets::Button::new("advanced")
            .size(button_size)
            .position(position)
            .ui(ui);

        if advanced_btn {
            if let Some(tileset_id) = draw_params.selected_tileset.clone() {
                //res = Some(EditorAction::OpenTilesetPropertiesWindow(tileset_id));
            }
        }

        res
    }
}