use std::collections::HashMap;

use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use core::math::color_from_hex_string;

use crate::map::MapObjectKind;
use crate::map::{Map, MapLayer, MapLayerKind, MapObject, MapProperty, MapTile, MapTileset};

const SPAWN_POINT_MAP_OBJECT_TYPE: &str = "spawn_point";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum TiledProperty {
    Bool { name: String, value: bool },
    Float { name: String, value: f32 },
    Int { name: String, value: i32 },
    String { name: String, value: String },
    Color { name: String, value: String },
    Object { name: String, value: i32 },
    File { name: String, value: String },
}

#[derive(Debug, Clone, Deserialize)]
pub struct TiledObject {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub object_type: String,
    pub x: f32,
    pub y: f32,
    pub height: f32,
    pub width: f32,
    pub visible: bool,
    pub rotation: f32,
    pub ellipse: Option<bool>,
    pub polygon: Option<Vec<TiledPolyPoint>>,
    pub properties: Option<Vec<TiledProperty>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TiledTileAttribute {
    pub id: u32,
    #[serde(rename = "type")]
    pub attribute: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TiledPolyPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TiledTileset {
    pub columns: i32,
    pub image: String,
    pub imagewidth: i32,
    pub imageheight: i32,
    pub margin: i32,
    pub name: String,
    #[serde(default, rename = "tiles")]
    pub tile_attributes: Option<Vec<TiledTileAttribute>>,
    #[serde(default)]
    pub properties: Option<Vec<TiledProperty>>,
    pub spacing: i32,
    pub tileheight: i32,
    pub tilewidth: i32,
    pub firstgid: u32,
    pub tilecount: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TiledLayer {
    pub name: String,
    pub visible: bool,
    #[serde(rename = "type")]
    pub layer_type: String,
    #[serde(default)]
    pub data: Vec<u32>,
    #[serde(default)]
    pub objects: Vec<TiledObject>,
    #[serde(default)]
    pub properties: Option<Vec<TiledProperty>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TiledMap {
    // Optional background color
    pub backgroundcolor: Option<String>,
    // Number of columns in the map grid
    pub width: u32,
    // Number of rows in the map grid
    pub height: u32,
    // Width of the individual tiles
    pub tilewidth: u32,
    // Height of the individual tiles
    pub tileheight: u32,
    // The JSON format version
    pub version: String,
    // The Tiled version used to create the map
    pub tiledversion: String,
    pub layers: Vec<TiledLayer>,
    pub tilesets: Vec<TiledTileset>,
    #[serde(default)]
    pub properties: Option<Vec<TiledProperty>>,
}

impl TiledMap {
    pub const TILE_LAYER_TYPE: &'static str = "tilelayer";

    pub const COLLISION_LAYER_PROP: &'static str = "collision";
    pub const TEXTURE_ID_PROP: &'static str = "texture_id";

    pub fn into_map(self) -> Map {
        let background_color = if let Some(background_color) = self.backgroundcolor {
            color_from_hex_string(&background_color)
        } else {
            Map::default_background_color()
        };

        let mut tilesets = HashMap::new();
        for tiled_tileset in self.tilesets {
            let texture_size = uvec2(
                tiled_tileset.imagewidth as u32,
                tiled_tileset.imageheight as u32,
            );

            let tile_size = vec2(
                tiled_tileset.tilewidth as f32,
                tiled_tileset.tileheight as f32,
            );

            let grid_size = uvec2(
                tiled_tileset.columns as u32,
                tiled_tileset.tilecount as u32 / tiled_tileset.columns as u32,
            );

            let mut tile_attributes: HashMap<u32, Vec<String>> = HashMap::new();
            if let Some(tiled_tile_attributes) = tiled_tileset.tile_attributes.as_ref() {
                for tiled_attr in tiled_tile_attributes {
                    tile_attributes.entry(tiled_attr.id).or_default();
                    tile_attributes
                        .get_mut(&tiled_attr.id)
                        .unwrap()
                        .push(tiled_attr.attribute.clone());
                }
            }

            let mut texture_id = None;
            let mut properties = HashMap::new();

            if let Some(tiled_props) = tiled_tileset.properties.as_ref() {
                for tiled_prop in tiled_props {
                    let (name, prop) = pair_from_tiled_prop(tiled_prop.clone());
                    if name == Self::TEXTURE_ID_PROP {
                        if let MapProperty::String(value) = &prop {
                            texture_id = Some(value.clone());
                            continue;
                        }
                    }

                    properties.insert(name, prop);
                }
            }

            let texture_id = texture_id.unwrap_or_else(|| {
                panic!(
                    "Tiled tileset '{}' needs a 'texture_id' property!",
                    &tiled_tileset.name
                )
            });

            let tile_subdivisions = MapTileset::default_tile_subdivisions();
            let subdivision_grid_size = grid_size * tile_subdivisions;

            let subtile_cnt = (subdivision_grid_size.x * subdivision_grid_size.y) as usize;
            let mut autotile_mask = vec![];
            autotile_mask.resize(subtile_cnt, false);

            let tileset = MapTileset {
                id: tiled_tileset.name.clone(),
                texture_id: texture_id.to_string(),
                texture_size,
                tile_size,
                grid_size,
                first_tile_id: tiled_tileset.firstgid,
                tile_cnt: tiled_tileset.tilecount,
                tile_subdivisions,
                autotile_mask,
                tile_attributes,
                properties,
            };

            tilesets.insert(tiled_tileset.name, tileset);
        }

        let mut spawn_points = Vec::new();
        let mut layers = HashMap::new();
        let mut draw_order = Vec::new();
        for tiled_layer in &self.layers {
            let mut tiles = Vec::new();
            for tile_id in tiled_layer.data.clone() {
                let res = if tile_id != 0 {
                    let tileset = tilesets
                        .iter()
                        .find_map(|(_, tileset)| {
                            if tile_id >= tileset.first_tile_id
                                && tile_id <= tileset.first_tile_id + tileset.tile_cnt
                            {
                                return Some(tileset);
                            }
                            None
                        })
                        .unwrap();

                    let tile_id = tile_id - tileset.first_tile_id;

                    let attributes = tileset
                        .tile_attributes
                        .get(&tile_id)
                        .cloned()
                        .unwrap_or_default();

                    let tile = MapTile {
                        tile_id,
                        tileset_id: tileset.id.clone(),
                        texture_id: tileset.texture_id.clone(),
                        texture_coords: tileset.get_texture_coords(tile_id),
                        attributes,
                    };

                    Some(tile)
                } else {
                    None
                };

                tiles.push(res);
            }

            let mut objects = Vec::new();
            for tiled_object in &tiled_layer.objects {
                let position = vec2(tiled_object.x, tiled_object.y);

                if tiled_object.object_type == *SPAWN_POINT_MAP_OBJECT_TYPE {
                    spawn_points.push(position);
                } else {
                    let mut properties = HashMap::new();
                    if let Some(tiled_props) = tiled_object.properties.clone() {
                        for tiled_prop in tiled_props {
                            let (name, prop) = pair_from_tiled_prop(tiled_prop);
                            properties.insert(name, prop);
                        }
                    }

                    let kind = MapObjectKind::from(tiled_object.object_type.clone());

                    let object = MapObject {
                        id: tiled_object.name.clone(),
                        kind,
                        position,
                        properties,
                    };

                    objects.push(object);
                }
            }

            let grid_size = uvec2(self.width, self.height);

            let mut has_collision = false;
            let mut properties = HashMap::new();
            if let Some(tiled_props) = &tiled_layer.properties {
                for tiled_prop in tiled_props {
                    let (name, prop) = pair_from_tiled_prop(tiled_prop.clone());
                    if name == Self::COLLISION_LAYER_PROP {
                        if let MapProperty::Bool(value) = &prop {
                            has_collision = *value;
                            continue;
                        }
                    }

                    properties.insert(name, prop);
                }
            }

            let kind = if tiled_layer.layer_type == *Self::TILE_LAYER_TYPE {
                MapLayerKind::TileLayer
            } else {
                MapLayerKind::ObjectLayer
            };

            let layer = MapLayer {
                id: tiled_layer.name.clone(),
                kind,
                has_collision,
                grid_size,
                tiles,
                objects,
                is_visible: tiled_layer.visible,
                properties,
            };

            draw_order.push(layer.id.clone());
            layers.insert(layer.id.clone(), layer);
        }

        let grid_size = uvec2(self.width, self.height);

        let mut properties = HashMap::new();
        if let Some(tiled_props) = self.properties {
            for tiled_prop in tiled_props {
                let (name, prop) = pair_from_tiled_prop(tiled_prop);
                properties.insert(name, prop);
            }
        }

        Map {
            background_color,
            background_layers: Vec::new(),
            world_offset: Vec2::ZERO,
            grid_size,
            tile_size: vec2(self.tilewidth as f32, self.tileheight as f32),
            layers,
            tilesets,
            draw_order,
            properties,
            spawn_points,
        }
    }
}

fn pair_from_tiled_prop(tiled_prop: TiledProperty) -> (String, MapProperty) {
    match tiled_prop {
        TiledProperty::Bool { name, value } => (name, MapProperty::Bool(value)),
        TiledProperty::Float { name, value } => (name, MapProperty::Float(value)),
        TiledProperty::Int { name, value } => (name, MapProperty::Int(value)),
        TiledProperty::String { name, value } => (name, MapProperty::String(value)),
        TiledProperty::Color { name, value } => {
            let color = color_from_hex_string(&value);
            (name, MapProperty::Color(color))
        }
        TiledProperty::Object { name, value } => (name, MapProperty::Int(value)),
        TiledProperty::File { name, value } => (name, MapProperty::String(value)),
    }
}
