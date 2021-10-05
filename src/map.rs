use std::{collections::HashMap, io, path::Path};

use macroquad::{color, experimental::collections::storage, prelude::*};

use serde::{Deserialize, Serialize};

use crate::{
    json::{self, TiledMap},
    math::URect,
    Resources,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(into = "json::MapDef", from = "json::MapDef")]
pub struct Map {
    pub name: String,
    #[serde(default = "Map::default_background_color", with = "json::ColorDef")]
    pub background_color: Color,
    #[serde(with = "json::def_vec2")]
    pub world_offset: Vec2,
    #[serde(with = "json::def_uvec2")]
    pub grid_size: UVec2,
    #[serde(with = "json::def_vec2")]
    pub tile_size: Vec2,
    pub layers: HashMap<String, MapLayer>,
    pub tilesets: HashMap<String, MapTileset>,
    #[serde(skip)]
    pub draw_order: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, MapProperty>,
}

impl Map {
    pub const DEFAULT_NAME: &'static str = "unnamed_map";

    pub const PLATFORM_TILE_ATTRIBUTE: &'static str = "jumpthrough";

    pub fn new(name: &str, tile_size: Vec2, grid_size: UVec2) -> Self {
        Map {
            name: name.to_string(),
            background_color: Self::default_background_color(),
            world_offset: Vec2::ZERO,
            grid_size,
            tile_size,
            layers: HashMap::new(),
            tilesets: HashMap::new(),
            draw_order: Vec::new(),
            properties: HashMap::new(),
        }
    }

    pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self, FileError> {
        let path = path.as_ref();

        let bytes = load_file(&path.to_string_lossy()).await?;
        let map = serde_json::from_slice(&bytes).unwrap();

        Ok(map)
    }

    pub async fn load_tiled<P: AsRef<Path>>(
        path: P,
        export_path: Option<P>,
    ) -> Result<Self, FileError> {
        let path = path.as_ref();

        let bytes = load_file(&path.to_string_lossy()).await?;
        let tiled_map: TiledMap = serde_json::from_slice(&bytes).unwrap();

        let name = map_name_from_path(path);
        let map = tiled_map.into_map(&name);

        if let Some(export_path) = export_path {
            map.save(export_path).unwrap();
        }

        Ok(map)
    }

    pub fn get_size(&self) -> Vec2 {
        vec2(
            self.grid_size.x as f32 * self.tile_size.x,
            self.grid_size.y as f32 * self.tile_size.y,
        )
    }

    pub fn to_grid(&self, rect: Rect) -> URect {
        let p = self.to_coords(rect.point());
        let w = ((rect.w / self.tile_size.x) as u32).clamp(0, self.grid_size.x - p.x - 1);
        let h = ((rect.h / self.tile_size.y) as u32).clamp(0, self.grid_size.y - p.y - 1);
        URect::new(p.x, p.y, w, h)
    }

    pub fn to_coords(&self, position: Vec2) -> UVec2 {
        let x = (((position.x - self.world_offset.x) / self.tile_size.x) as u32)
            .clamp(0, self.grid_size.x - 1);
        let y = (((position.y - self.world_offset.y) / self.tile_size.y) as u32)
            .clamp(0, self.grid_size.y - 1);
        uvec2(x, y)
    }

    pub fn to_index(&self, coords: UVec2) -> usize {
        ((coords.y * self.grid_size.x) + coords.x) as usize
    }

    pub fn to_position(&self, point: UVec2) -> Vec2 {
        vec2(
            point.x as f32 * self.tile_size.x + self.world_offset.x,
            point.y as f32 * self.tile_size.y + self.world_offset.y,
        )
    }

    pub fn get_tile(&self, layer_id: &str, x: u32, y: u32) -> &Option<MapTile> {
        let layer = self
            .layers
            .get(layer_id)
            .unwrap_or_else(|| panic!("No layer with id '{}'!", layer_id));

        if x >= self.grid_size.x || y >= self.grid_size.y {
            return &None;
        };

        let i = (y * self.grid_size.x + x) as usize;
        &layer.tiles[i]
    }

    pub fn get_tiles(&self, layer_id: &str, rect: Option<URect>) -> MapTileIterator {
        let rect = rect.unwrap_or_else(|| URect::new(0, 0, self.grid_size.x, self.grid_size.y));
        let layer = self
            .layers
            .get(layer_id)
            .unwrap_or_else(|| panic!("No layer with id '{}'!", layer_id));

        MapTileIterator::new(layer, rect)
    }

    pub fn get_collisions(&self, collider: Rect) -> Vec<Vec2> {
        let collider = Rect::new(
            collider.x - self.tile_size.x,
            collider.y - self.tile_size.y,
            collider.w + self.tile_size.x * 2.0,
            collider.h + self.tile_size.y * 2.0,
        );

        let rect = self.to_grid(collider);
        let mut collisions = Vec::new();
        for layer in self.layers.values() {
            if layer.is_visible && layer.has_collision {
                for (x, y, tile) in self.get_tiles(&layer.id, Some(rect)) {
                    if tile.is_some() {
                        let tile_position = self.to_position(uvec2(x, y));
                        if Rect::new(
                            tile_position.x,
                            tile_position.y,
                            self.tile_size.x,
                            self.tile_size.y,
                        )
                        .overlaps(&collider)
                        {
                            collisions.push(tile_position);
                        }
                    }
                }
            }
        }
        collisions
    }

    pub fn draw(&self, rect: Option<URect>) {
        let rect = rect.unwrap_or_else(|| URect::new(0, 0, self.grid_size.x, self.grid_size.y));
        draw_rectangle(
            self.world_offset.x + (rect.x as f32 * self.tile_size.x),
            self.world_offset.y + (rect.y as f32 * self.tile_size.y),
            rect.w as f32 * self.tile_size.x,
            rect.h as f32 * self.tile_size.y,
            self.background_color,
        );

        let resources = storage::get::<Resources>();
        for layer_id in &self.draw_order {
            if let Some(layer) = self.layers.get(layer_id) {
                if layer.is_visible && layer.kind == MapLayerKind::TileLayer {
                    for (x, y, tile) in self.get_tiles(layer_id, Some(rect)) {
                        if let Some(tile) = tile {
                            let world_position = self.world_offset
                                + vec2(x as f32 * self.tile_size.x, y as f32 * self.tile_size.y);

                            let texture = resources
                                .textures
                                .get(&tile.texture_id)
                                .cloned()
                                .unwrap_or_else(|| {
                                    panic!("No texture with id '{}'!", tile.texture_id)
                                });

                            draw_texture_ex(
                                texture,
                                world_position.x,
                                world_position.y,
                                color::WHITE,
                                DrawTextureParams {
                                    source: Some(Rect::new(
                                        tile.texture_coords.x, // + 0.1,
                                        tile.texture_coords.y, // + 0.1,
                                        self.tile_size.x,      // - 0.2,
                                        self.tile_size.y,      // - 0.2,
                                    )),
                                    dest_size: Some(vec2(self.tile_size.x, self.tile_size.y)),
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn get_layer_kind(&self, layer_id: &str) -> Option<MapLayerKind> {
        if let Some(layer) = self.layers.get(layer_id) {
            return Some(layer.kind);
        }

        None
    }

    pub fn default_background_color() -> Color {
        Color::new(0.0, 0.0, 0.0, 0.0)
    }

    #[cfg(any(target_family = "unix", target_family = "windows"))]
    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    #[cfg(target_family = "wasm")]
    pub fn save<P: AsRef<Path>>(&self, _: P) -> Result<()> {
        Ok(())
    }
}

pub struct MapTileIterator<'a> {
    rect: URect,
    current: (u32, u32),
    layer: &'a MapLayer,
}

impl<'a> MapTileIterator<'a> {
    fn new(layer: &'a MapLayer, rect: URect) -> Self {
        let current = (rect.x, rect.y);
        MapTileIterator {
            layer,
            rect,
            current,
        }
    }
}

impl<'a> Iterator for MapTileIterator<'a> {
    type Item = (u32, u32, &'a Option<MapTile>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = if self.current.0 + 1 >= self.rect.x + self.rect.w {
            (self.rect.x, self.current.1 + 1)
        } else {
            (self.current.0 + 1, self.current.1)
        };

        let i = (self.current.1 * self.layer.grid_size.x + self.current.0) as usize;
        if self.current.1 >= self.rect.y + self.rect.h || i >= self.layer.tiles.len() {
            return None;
        }

        let res = Some((self.current.0, self.current.1, &self.layer.tiles[i]));

        self.current = next;

        res
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectLayerKind {
    None,
    Items,
    SpawnPoints,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapLayerKind {
    TileLayer,
    ObjectLayer(ObjectLayerKind),
}

impl Default for MapLayerKind {
    fn default() -> Self {
        MapLayerKind::TileLayer
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLayer {
    pub id: String,
    pub kind: MapLayerKind,
    #[serde(default, rename = "collision")]
    pub has_collision: bool,
    #[serde(with = "json::def_uvec2")]
    pub grid_size: UVec2,
    pub tiles: Vec<Option<MapTile>>,
    pub objects: Vec<MapObject>,
    #[serde(default)]
    pub is_visible: bool,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, MapProperty>,
}

impl MapLayer {
    pub fn new(id: &str, kind: MapLayerKind) -> Self {
        MapLayer {
            id: id.to_string(),
            kind,
            ..Default::default()
        }
    }
}

impl Default for MapLayer {
    fn default() -> Self {
        MapLayer {
            id: "".to_string(),
            has_collision: false,
            kind: MapLayerKind::TileLayer,
            grid_size: UVec2::ZERO,
            tiles: Vec::new(),
            objects: Vec::new(),
            is_visible: true,
            properties: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapTile {
    pub tile_id: u32,
    pub tileset_id: String,
    pub texture_id: String,
    #[serde(with = "json::def_vec2")]
    pub texture_coords: Vec2,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapObject {
    pub name: String,
    #[serde(with = "json::def_vec2")]
    pub position: Vec2,
    #[serde(
        default,
        with = "json::opt_vec2",
        skip_serializing_if = "Option::is_none"
    )]
    pub size: Option<Vec2>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, MapProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MapProperty {
    Bool {
        value: bool,
    },
    Float {
        value: f32,
    },
    Int {
        value: i32,
    },
    String {
        value: String,
    },
    Color {
        #[serde(with = "json::ColorDef")]
        value: Color,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapTileset {
    pub id: String,
    pub texture_id: String,
    #[serde(with = "json::def_uvec2")]
    pub texture_size: UVec2,
    #[serde(with = "json::def_vec2")]
    pub tile_size: Vec2,
    #[serde(with = "json::def_uvec2")]
    pub grid_size: UVec2,
    pub first_tile_id: u32,
    pub tile_cnt: u32,
    #[serde(
        default = "MapTileset::default_tile_subdivisions",
        with = "json::def_uvec2"
    )]
    pub tile_subdivisions: UVec2,
    pub autotile_mask: Vec<bool>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tile_attributes: HashMap<u32, Vec<String>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, MapProperty>,
}

impl MapTileset {
    pub fn new(
        id: &str,
        texture_id: &str,
        texture_size: UVec2,
        tile_size: Vec2,
        first_tile_id: u32,
    ) -> Self {
        let grid_size = uvec2(
            texture_size.x / tile_size.x as u32,
            texture_size.y / tile_size.y as u32,
        );

        let tile_subdivisions = Self::default_tile_subdivisions();

        let subtile_grid_size = grid_size * tile_subdivisions;

        let subtile_cnt = (subtile_grid_size.x * subtile_grid_size.y) as usize;

        let mut autotile_mask = vec![];
        autotile_mask.resize(subtile_cnt, false);

        MapTileset {
            id: id.to_string(),
            texture_id: texture_id.to_string(),
            texture_size,
            tile_size,
            grid_size,
            first_tile_id,
            tile_cnt: grid_size.x * grid_size.y,
            tile_subdivisions,
            autotile_mask,
            tile_attributes: HashMap::new(),
            properties: HashMap::new(),
        }
    }

    pub fn get_texture_coords(&self, tile_id: u32) -> Vec2 {
        let x = (tile_id % self.grid_size.x) as f32 * self.tile_size.x;
        let y = (tile_id / self.grid_size.x) as f32 * self.tile_size.y;
        vec2(x, y)
    }

    pub fn default_tile_subdivisions() -> UVec2 {
        uvec2(3, 3)
    }
}

pub fn map_name_from_path<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();
    if let Some(os_str) = path.file_name() {
        let str = os_str.to_str().unwrap();
        let mut split = str.split('.').collect::<Vec<_>>();
        let split_len = split.len();
        if split_len > 0 {
            if split_len > 1 {
                split.pop();
            }

            return split.join(".");
        }
    }

    Map::DEFAULT_NAME.to_string()
}
