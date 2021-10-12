use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{widgets, Ui},
};

use super::{EditorAction, EditorContext, GuiResources, Map, ToolbarElement, ToolbarElementParams};

use crate::{editor::gui::ELEMENT_MARGIN, Resources};

pub struct TilesetDetailsElement {
    params: ToolbarElementParams,
}

impl TilesetDetailsElement {
    pub fn new() -> Self {
        let params = ToolbarElementParams {
            header: None,
            has_margins: true,
            ..Default::default()
        };

        TilesetDetailsElement { params }
    }
}

impl ToolbarElement for TilesetDetailsElement {
    fn get_params(&self) -> &ToolbarElementParams {
        &self.params
    }

    fn draw(
        &mut self,
        ui: &mut Ui,
        size: Vec2,
        map: &Map,
        ctx: &EditorContext,
    ) -> Option<EditorAction> {
        let mut res = None;

        let mut position = Vec2::ZERO;

        if let Some(tileset_id) = &ctx.selected_tileset {
            let tileset = map.tilesets.get(tileset_id).unwrap();

            let texture_entry = {
                let resources = storage::get::<Resources>();
                *resources
                    .textures
                    .get(&tileset.texture_id)
                    .unwrap()
            };

            let grid_size = vec2(tileset.grid_size.x as f32, tileset.grid_size.y as f32);

            let scaled_width = size.x;
            let scaled_height = (scaled_width / texture_entry.texture.width()) * texture_entry.texture.height();

            let scaled_tile_size = vec2(scaled_width / grid_size.x, scaled_height / grid_size.y);

            widgets::Texture::new(texture_entry.texture)
                .position(position)
                .size(scaled_width, scaled_height)
                .ui(ui);

            {
                let gui_resources = storage::get::<GuiResources>();
                ui.push_skin(&gui_resources.editor_skins.tileset_grid);
            }

            for y in 0..tileset.grid_size.y {
                for x in 0..tileset.grid_size.x {
                    let tile_id = y * tileset.grid_size.x + x;

                    let is_selected = if let Some(selected) = ctx.selected_tile {
                        selected == tile_id
                    } else {
                        false
                    };

                    if is_selected {
                        let gui_resources = storage::get::<GuiResources>();
                        ui.push_skin(&gui_resources.editor_skins.tileset_grid_selected);
                    }

                    let position = vec2(x as f32, y as f32) * scaled_tile_size;

                    let button = widgets::Button::new("")
                        .size(scaled_tile_size)
                        .position(position)
                        .ui(ui);

                    if button {
                        res = Some(EditorAction::SelectTile {
                            id: tile_id,
                            tileset_id: tileset.id.clone(),
                        });
                    }

                    if is_selected {
                        ui.pop_skin();
                    }
                }
            }

            ui.pop_skin();

            position.y += scaled_height + ELEMENT_MARGIN;
        }

        res
    }

    fn is_drawn(&self, _map: &Map, ctx: &EditorContext) -> bool {
        ctx.selected_tileset.is_some()
    }
}

impl Default for TilesetDetailsElement {
    fn default() -> Self {
        Self::new()
    }
}
