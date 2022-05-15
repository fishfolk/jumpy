pub mod tiled;

use std::{collections::HashMap, iter::FromIterator};

use crate::prelude::*;

use serde::{Deserialize, Serialize};

use crate::map::{
    Map, MapBackgroundLayer, MapLayer, MapLayerKind, MapObject, MapProperty, MapTile, MapTileset,
};

pub use tiled::TiledMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MapDef {
    #[serde(default = "Map::default_background_color")]
    pub background_color: Color,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub background_layers: Vec<MapBackgroundLayer>,
    #[serde(with = "crate::parsing::vec2_def", default)]
    pub world_offset: Vec2,
    pub grid_size: Size<u32>,
    pub tile_size: Size<f32>,
    pub layers: Vec<MapLayerDef>,
    pub tilesets: Vec<MapTileset>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, MapProperty>,
    #[serde(default, with = "crate::parsing::vec2_vec")]
    pub spawn_points: Vec<Vec2>,
}

impl From<Map> for MapDef {
    fn from(other: Map) -> MapDef {
        let layers = other
            .draw_order
            .iter()
            .filter_map(|layer_id| {
                if let Some(layer) = other.layers.get(layer_id) {
                    let (tiles, objects) = match &layer.kind {
                        MapLayerKind::TileLayer => {
                            let tiles = layer
                                .tiles
                                .iter()
                                .map(|opt| match opt {
                                    Some(tile) => {
                                        let tileset = other
                                            .tilesets
                                            .get(&tile.tileset_id)
                                            .unwrap_or_else(|| {
                                                panic!(
                                                    "Unable to find tileset with id '{}'!",
                                                    tile.tileset_id
                                                )
                                            });
                                        tile.tile_id + tileset.first_tile_id
                                    }
                                    _ => 0,
                                })
                                .collect();

                            (Some(tiles), None)
                        }
                        MapLayerKind::ObjectLayer => {
                            let objects = layer.objects.clone();

                            (None, Some(objects))
                        }
                    };

                    let layer = MapLayerDef {
                        id: layer.id.clone(),
                        kind: layer.kind,
                        has_collision: layer.has_collision,
                        objects,
                        tiles,
                        is_visible: layer.is_visible,
                        properties: layer.properties.clone(),
                    };

                    return Some(layer);
                }

                None
            })
            .collect();

        let tilesets = other
            .tilesets
            .into_iter()
            .map(|(_, tileset)| tileset)
            .collect();

        MapDef {
            background_color: other.background_color,
            background_layers: other.background_layers,
            world_offset: other.world_offset,
            grid_size: other.grid_size,
            tile_size: other.tile_size,
            layers,
            tilesets,
            properties: other.properties,
            spawn_points: other.spawn_points,
        }
    }
}

impl From<MapDef> for Map {
    fn from(def: MapDef) -> Self {
        let tilesets = HashMap::from_iter(
            def.tilesets
                .clone()
                .into_iter()
                .map(|tileset| (tileset.id.clone(), tileset)),
        );

        let draw_order = def.layers.iter().map(|layer| layer.id.clone()).collect();

        let layers = HashMap::from_iter(def.layers.iter().map(|layer| {
            let tiles = layer
                .tiles
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|tile_id| {
                    if tile_id == 0 {
                        None
                    } else {
                        let tile = match tilesets.iter().find(|(_, tileset)| {
                            tile_id >= tileset.first_tile_id
                                && tile_id < tileset.first_tile_id + tileset.tile_cnt
                        }) {
                            Some((_, tileset)) => {
                                let tile_id = tile_id - tileset.first_tile_id;
                                let mut attributes = Vec::new();
                                if let Some(tile_attributes) =
                                    tileset.tile_attributes.get(&tile_id).cloned()
                                {
                                    for attribute in tile_attributes {
                                        attributes.push(attribute);
                                    }
                                }

                                let tile = MapTile {
                                    tile_id,
                                    texture: None,
                                    tileset_id: tileset.id.clone(),
                                    texture_id: tileset.texture_id.clone(),
                                    texture_coords: tileset.get_texture_coords(tile_id),
                                    attributes,
                                };

                                Some(tile)
                            }
                            _ => None,
                        };
                        assert!(
                            tile.is_some(),
                            "Unable to determine tileset from tile_id '{}'",
                            tile_id
                        );
                        tile
                    }
                })
                .collect();

            let objects = layer.objects.clone().unwrap_or_default();

            let layer = MapLayer {
                id: layer.id.clone(),
                kind: layer.kind,
                has_collision: layer.has_collision,
                grid_size: def.grid_size,
                tiles,
                objects,
                is_visible: layer.is_visible,
                properties: layer.properties.clone(),
            };

            (layer.id.clone(), layer)
        }));

        Map {
            background_color: def.background_color,
            background_layers: def.background_layers,
            world_offset: def.world_offset,
            grid_size: def.grid_size,
            tile_size: def.tile_size,
            layers,
            tilesets,
            draw_order,
            properties: def.properties,
            spawn_points: def.spawn_points,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLayerDef {
    pub id: String,
    pub kind: MapLayerKind,
    #[serde(default)]
    pub has_collision: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tiles: Option<Vec<u32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objects: Option<Vec<MapObject>>,
    #[serde(default)]
    pub is_visible: bool,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, MapProperty>,
}

impl Default for MapLayerDef {
    fn default() -> Self {
        MapLayerDef {
            id: "".to_string(),
            kind: MapLayerKind::TileLayer,
            has_collision: false,
            tiles: Some(Vec::new()),
            objects: None,
            is_visible: true,
            properties: HashMap::new(),
        }
    }
}
