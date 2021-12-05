use macroquad::{color, experimental::collections::storage, prelude::*};

use super::{EditorAction, EditorContext, EditorTool, EditorToolParams};

use crate::{
    editor::EditorCamera,
    map::{Map, MapLayerKind},
    Resources,
};

#[derive(Default)]
pub struct TilePlacementTool {
    params: EditorToolParams,
}

impl TilePlacementTool {
    pub fn new() -> Self {
        let params = EditorToolParams {
            name: "Place Tiles".to_string(),
            is_continuous: true,
            ..Default::default()
        };

        TilePlacementTool { params }
    }
}

impl EditorTool for TilePlacementTool {
    fn get_params(&self) -> &EditorToolParams {
        &self.params
    }

    fn get_action(&mut self, map: &Map, ctx: &EditorContext) -> Option<EditorAction> {
        let cursor_world_position = scene::find_node_by_type::<EditorCamera>()
            .unwrap()
            .to_world_space(ctx.cursor_position);

        if map.contains(cursor_world_position) {
            if let Some(layer_id) = &ctx.selected_layer {
                let camera = scene::find_node_by_type::<EditorCamera>().unwrap();
                let world_position = camera.to_world_space(ctx.cursor_position);

                if let Some(tileset_id) = &ctx.selected_tileset {
                    if let Some(tile_id) = ctx.selected_tile {
                        let coords = map.to_coords(world_position);

                        return Some(EditorAction::PlaceTile {
                            id: tile_id,
                            layer_id: layer_id.clone(),
                            tileset_id: tileset_id.clone(),
                            coords,
                        });
                    }
                }
            }
        }

        None
    }

    fn is_available(&self, map: &Map, ctx: &EditorContext) -> bool {
        if let Some(layer_id) = &ctx.selected_layer {
            let layer = map.layers.get(layer_id).unwrap();
            return layer.kind == MapLayerKind::TileLayer;
        }

        false
    }

    fn draw_cursor(&mut self, map: &Map, ctx: &EditorContext) -> Option<EditorAction> {
        let cursor_world_position = scene::find_node_by_type::<EditorCamera>()
            .unwrap()
            .to_world_space(ctx.cursor_position);

        if map.contains(cursor_world_position) {
            if let Some(layer_id) = &ctx.selected_layer {
                let layer = map.layers.get(layer_id).unwrap();

                if layer.kind == MapLayerKind::TileLayer {
                    if let Some(tileset_id) = &ctx.selected_tileset {
                        if let Some(tile_id) = ctx.selected_tile {
                            let tileset = map.tilesets.get(tileset_id).unwrap();

                            let cursor_world_position = scene::find_node_by_type::<EditorCamera>()
                                .unwrap()
                                .to_world_space(ctx.cursor_position);

                            let coords = map.to_coords(cursor_world_position);
                            let position = map.to_position(coords);

                            let texture_coords = tileset.get_texture_coords(tile_id);
                            let texture = {
                                let resources = storage::get::<Resources>();
                                let res = resources.textures.get(&tileset.texture_id).unwrap();
                                res.texture
                            };

                            let source_rect = Rect::new(
                                texture_coords.x,
                                texture_coords.y,
                                map.tile_size.x,
                                map.tile_size.y,
                            );

                            draw_texture_ex(
                                texture,
                                position.x,
                                position.y,
                                color::WHITE,
                                DrawTextureParams {
                                    dest_size: Some(map.tile_size),
                                    source: Some(source_rect),
                                    ..Default::default()
                                },
                            )
                        }
                    }
                }
            }
        }

        None
    }
}

#[derive(Default)]
pub struct ObjectPlacementTool {
    params: EditorToolParams,
}

impl ObjectPlacementTool {
    pub fn new() -> Self {
        let params = EditorToolParams {
            name: "Place Objects".to_string(),
            ..Default::default()
        };

        ObjectPlacementTool { params }
    }
}

impl EditorTool for ObjectPlacementTool {
    fn get_params(&self) -> &EditorToolParams {
        &self.params
    }

    fn get_action(&mut self, map: &Map, ctx: &EditorContext) -> Option<EditorAction> {
        let cursor_world_position = scene::find_node_by_type::<EditorCamera>()
            .unwrap()
            .to_world_space(ctx.cursor_position);

        if map.contains(cursor_world_position) {
            if let Some(layer_id) = ctx.selected_layer.clone() {
                let layer = map.layers.get(&layer_id).unwrap();

                if layer.kind == MapLayerKind::ObjectLayer {
                    let mut position = scene::find_node_by_type::<EditorCamera>()
                        .unwrap()
                        .to_world_space(ctx.cursor_position);

                    let rect = Rect::new(
                        map.world_offset.x,
                        map.world_offset.y,
                        map.grid_size.x as f32 * map.tile_size.x,
                        map.grid_size.y as f32 * map.tile_size.y,
                    );

                    if ctx.should_snap_to_grid {
                        let coords = map.to_coords(position);
                        position = map.to_position(coords);
                    }

                    if rect.contains(position) {
                        let action = EditorAction::OpenCreateObjectWindow { position, layer_id };

                        return Some(action);
                    }
                }
            }
        }

        None
    }

    fn is_available(&self, map: &Map, ctx: &EditorContext) -> bool {
        if let Some(layer_id) = &ctx.selected_layer {
            let layer = map.layers.get(layer_id).unwrap();
            return layer.kind == MapLayerKind::ObjectLayer;
        }

        false
    }
}
