use ff_core::prelude::*;

use super::{EditorAction, EditorContext, EditorTool, EditorToolParams};

use crate::editor::EditorCamera;

use ff_core::macroquad::experimental::scene;
use ff_core::map::{Map, MapLayerKind};

#[derive(Default)]
pub struct TilePlacementTool {
    params: EditorToolParams,
}

impl TilePlacementTool {
    pub fn new() -> Self {
        let params = EditorToolParams {
            name: "Place Tiles".to_string(),
            icon_texture_id: "tile_placement_tool_icon".to_string(),
            is_continuous: true,
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

    fn update(&mut self, map: &Map, ctx: &EditorContext) -> Option<EditorAction> {
        #[allow(unused_mut)]
        let mut res = None;

        if self.is_available(map, ctx) {
            if let Some(tileset_id) = &ctx.selected_tileset {
                let _tileset = map.tilesets.get(tileset_id).unwrap();

                // Do autotile resolution here and set `res` to an `EditorAction::SelectTile` if
                // selected tile should be changed according to context.
            }
        }

        res
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
                            let texture = get_texture(&tileset.texture_id).texture;

                            let source_rect = Rect::new(
                                texture_coords.x,
                                texture_coords.y,
                                map.tile_size.width,
                                map.tile_size.height,
                            );

                            draw_texture(
                                position.x,
                                position.y,
                                texture,
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
            icon_texture_id: "object_placement_tool_icon".to_string(),
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
                        map.grid_size.width as f32 * map.tile_size.width,
                        map.grid_size.height as f32 * map.tile_size.height,
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

pub struct SpawnPointPlacementTool {
    params: EditorToolParams,
}

impl SpawnPointPlacementTool {
    pub fn new() -> Self {
        let params = EditorToolParams {
            name: "Place Spawn Point".to_string(),
            icon_texture_id: "spawn_point_placement_tool_icon".to_string(),
            ..Default::default()
        };

        SpawnPointPlacementTool { params }
    }
}

impl EditorTool for SpawnPointPlacementTool {
    fn get_params(&self) -> &EditorToolParams {
        &self.params
    }

    fn get_action(&mut self, _map: &Map, ctx: &EditorContext) -> Option<EditorAction> {
        // TODO: Snap to grid

        let cursor_world_position = scene::find_node_by_type::<EditorCamera>()
            .unwrap()
            .to_world_space(ctx.cursor_position);

        let texture = get_texture("spawn_point_icon");
        let texture_size = texture.size();
        let offset = vec2(texture_size.width / 2.0, texture_size.height);

        let action = EditorAction::CreateSpawnPoint(cursor_world_position - offset);

        Some(action)
    }
}
