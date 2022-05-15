use std::borrow::BorrowMut;
use std::fs;
use std::slice::Iter;
use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

mod decoration;

pub use decoration::*;

use crate::error::ErrorKind;
use crate::prelude::*;
use crate::result::Result;

#[cfg(feature = "macroquad-backend")]
use crate::gui::combobox::ComboBoxValue;

use crate::parsing::{self, TiledMap};
use crate::resources::DEFAULT_RESOURCE_FILE_EXTENSION;

use crate::texture::get_texture;

pub type MapProperty = crate::parsing::GenericParam;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapBackgroundLayer {
    pub texture_id: String,
    pub depth: f32,
    #[serde(with = "crate::parsing::vec2_def")]
    pub offset: Vec2,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(into = "parsing::MapDef", from = "parsing::MapDef")]
pub struct Map {
    #[serde(
        default = "Map::default_background_color",
        with = "crate::parsing::ColorDef"
    )]
    pub background_color: Color,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub background_layers: Vec<MapBackgroundLayer>,
    #[serde(with = "crate::parsing::def_vec2")]
    pub world_offset: Vec2,
    pub grid_size: Size<u32>,
    pub tile_size: Size<f32>,
    pub layers: HashMap<String, MapLayer>,
    pub tilesets: HashMap<String, MapTileset>,
    #[serde(skip)]
    pub draw_order: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, MapProperty>,
    #[serde(default, with = "crate::parsing::vec2_vec")]
    pub spawn_points: Vec<Vec2>,
}

impl Map {
    pub const PLATFORM_TILE_ATTRIBUTE: &'static str = "jumpthrough";

    // Padding added to colliders for collision checks since the collision system stops movement
    // before collision is registered, if not.
    pub const COLLIDER_PADDING: f32 = 8.0;

    const FLATTENED_BACKGROUND_PADDING_X: f32 = 100.0;
    const FLATTENED_BACKGROUND_PADDING_Y: f32 = 100.0;

    pub fn new(tile_size: Vec2, grid_size: UVec2) -> Self {
        Map {
            background_color: Self::default_background_color(),
            background_layers: Vec::new(),
            world_offset: Vec2::ZERO,
            grid_size: grid_size.into(),
            tile_size: tile_size.into(),
            layers: HashMap::new(),
            tilesets: HashMap::new(),
            draw_order: Vec::new(),
            properties: HashMap::new(),
            spawn_points: Vec::new(),
        }
    }

    pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let extension = path.as_ref().extension().unwrap().to_str().unwrap();

        let bytes = read_from_file(&path).await?;

        let map = deserialize_bytes_by_extension(extension, &bytes).unwrap();

        Ok(map)
    }

    pub async fn load_tiled<P: AsRef<Path>>(path: P, export_path: Option<P>) -> Result<Self> {
        let bytes = read_from_file(path).await?;

        let tiled_map: TiledMap = deserialize_json_bytes(&bytes).unwrap();

        let map = tiled_map.into_map();

        if let Some(export_path) = export_path {
            map.save(export_path).unwrap();
        }

        Ok(map)
    }

    pub fn get_size(&self) -> Size<f32> {
        Size::new(
            self.grid_size.width as f32 * self.tile_size.width,
            self.grid_size.height as f32 * self.tile_size.height,
        )
    }

    pub fn contains(&self, position: Vec2) -> bool {
        let map_size = Size::from(self.grid_size.as_uvec2().as_vec2() * self.tile_size.as_vec2());
        let rect = Rect::new(
            self.world_offset.x,
            self.world_offset.y,
            map_size.width,
            map_size.height,
        );
        rect.contains(position)
    }

    pub fn to_grid(&self, rect: &Rect) -> URect {
        let p = self.to_coords(rect.point());
        let w =
            ((rect.width / self.tile_size.width) as u32).clamp(0, self.grid_size.width - p.x - 1);
        let h = ((rect.height / self.tile_size.height) as u32)
            .clamp(0, self.grid_size.height - p.y - 1);
        URect::new(p.x, p.y, w, h)
    }

    pub fn to_coords(&self, position: Vec2) -> UVec2 {
        let x = (((position.x - self.world_offset.x) / self.tile_size.width) as u32)
            .clamp(0, self.grid_size.width - 1);
        let y = (((position.y - self.world_offset.y) / self.tile_size.height) as u32)
            .clamp(0, self.grid_size.height - 1);
        uvec2(x, y)
    }

    pub fn to_index(&self, coords: UVec2) -> usize {
        ((coords.y * self.grid_size.width) + coords.x) as usize
    }

    pub fn to_position(&self, point: UVec2) -> Vec2 {
        vec2(
            (point.x as f32 * self.tile_size.width) + self.world_offset.x,
            (point.y as f32 * self.tile_size.height) + self.world_offset.y,
        )
    }

    pub fn get_tile(&self, layer_id: &str, x: u32, y: u32) -> &Option<MapTile> {
        let layer = self
            .layers
            .get(layer_id)
            .unwrap_or_else(|| panic!("No layer with id '{}'!", layer_id));

        if x >= self.grid_size.width || y >= self.grid_size.height {
            return &None;
        };

        let i = (y * self.grid_size.width + x) as usize;
        &layer.tiles[i]
    }

    pub fn get_tiles(&self, layer_id: &str, rect: Option<URect>) -> MapTileIterator {
        let rect =
            rect.unwrap_or_else(|| URect::new(0, 0, self.grid_size.width, self.grid_size.height));
        let layer = self
            .layers
            .get(layer_id)
            .unwrap_or_else(|| panic!("No layer with id '{}'!", layer_id));

        MapTileIterator::new(layer, rect)
    }

    pub fn get_collisions(&self, collider: &Rect, should_ignore_platforms: bool) -> Vec<Rect> {
        let collider = Rect::new(
            collider.x - Self::COLLIDER_PADDING,
            collider.y - Self::COLLIDER_PADDING,
            collider.width + Self::COLLIDER_PADDING * 2.0,
            collider.height + Self::COLLIDER_PADDING * 2.0,
        );

        let grid = self.to_grid(&Rect::new(
            collider.x - self.tile_size.width,
            collider.y - self.tile_size.height,
            collider.width + self.tile_size.width * 2.0,
            collider.height + self.tile_size.height * 2.0,
        ));

        let mut collisions = Vec::new();

        let platform_attr = Self::PLATFORM_TILE_ATTRIBUTE.to_string();

        for layer in self.layers.values() {
            if layer.is_visible && layer.has_collision {
                for (x, y, tile) in self.get_tiles(&layer.id, Some(grid)) {
                    if let Some(tile) = tile {
                        if !(should_ignore_platforms && tile.attributes.contains(&platform_attr)) {
                            let tile_position = self.to_position(uvec2(x, y));

                            let tile_rect = Rect::new(
                                tile_position.x,
                                tile_position.y,
                                self.tile_size.width,
                                self.tile_size.height,
                            );

                            if tile_rect.overlaps(&collider) {
                                collisions.push(tile_rect);
                            }
                        }
                    }
                }
            }
        }

        collisions
    }

    pub fn is_collision_at(&self, position: Vec2, should_ignore_platforms: bool) -> bool {
        let index = {
            let coords = self.to_coords(position);
            self.to_index(coords)
        };

        for layer in self.layers.values() {
            if layer.is_visible && layer.has_collision {
                if let Some(Some(tile)) = layer.tiles.get(index) {
                    return !(should_ignore_platforms
                        && tile
                            .attributes
                            .contains(&Self::PLATFORM_TILE_ATTRIBUTE.to_string()));
                }
            }
        }

        false
    }

    fn background_parallax(texture: Texture2D, depth: f32, camera_position: Vec2) -> Rect {
        let size = texture.size();

        let dest_rect = Rect::new(0.0, 0.0, size.width, size.height);
        let parallax_w = size.width * 0.5;

        let mut dest_rect2 = Rect::new(
            -parallax_w,
            -parallax_w,
            size.width + parallax_w * 2.,
            size.height + parallax_w * 2.,
        );

        let parallax_x = camera_position.x / dest_rect.width - 0.3;
        let parallax_y = camera_position.y / dest_rect.height * 0.6 - 0.5;

        dest_rect2.x += parallax_w * parallax_x * depth;
        dest_rect2.y += parallax_w * parallax_y * depth;

        dest_rect2
    }

    pub fn draw_background(
        &self,
        rect: Option<URect>,
        camera_position: Vec2,
        is_parallax_disabled: bool,
    ) {
        let rect =
            rect.unwrap_or_else(|| URect::new(0, 0, self.grid_size.width, self.grid_size.height));

        draw_rectangle(
            self.world_offset.x,
            self.world_offset.y,
            rect.width as f32 * self.tile_size.width,
            rect.height as f32 * self.tile_size.height,
            self.background_color,
        );

        {
            for layer in &self.background_layers {
                let texture = get_texture(&layer.texture_id);

                let dest_rect = if is_parallax_disabled {
                    let map_size = Size::from(self.grid_size.as_uvec2().as_vec2()) * self.tile_size;

                    let size = texture.size();

                    let width = map_size.width + (Self::FLATTENED_BACKGROUND_PADDING_X * 2.0);
                    let height = (width / size.width) * size.height;

                    Rect::new(
                        self.world_offset.x - Self::FLATTENED_BACKGROUND_PADDING_X,
                        self.world_offset.y - Self::FLATTENED_BACKGROUND_PADDING_Y,
                        width,
                        height,
                    )
                } else {
                    let mut dest_rect =
                        Self::background_parallax(texture, layer.depth, camera_position);
                    dest_rect.x += layer.offset.x;
                    dest_rect.y += layer.offset.y;
                    dest_rect
                };

                draw_texture(
                    dest_rect.x,
                    dest_rect.y,
                    texture,
                    DrawTextureParams {
                        dest_size: Some(Size::new(dest_rect.width, dest_rect.height)),
                        ..Default::default()
                    },
                )
            }
        }
    }

    /// This will draw the map
    pub fn draw<P: Into<Option<Vec2>>>(&self, rect: Option<URect>, camera_position: P) {
        if let Some(camera_position) = camera_position.into() {
            self.draw_background(rect, camera_position, false);
        }

        let rect =
            rect.unwrap_or_else(|| URect::new(0, 0, self.grid_size.width, self.grid_size.height));

        let mut draw_order = self.draw_order.clone();
        draw_order.reverse();

        for layer_id in draw_order {
            if let Some(layer) = self.layers.get(&layer_id) {
                if layer.is_visible && layer.kind == MapLayerKind::TileLayer {
                    for (x, y, tile) in self.get_tiles(&layer_id, Some(rect)) {
                        if let Some(tile) = tile {
                            let world_position = self.world_offset
                                + vec2(
                                    x as f32 * self.tile_size.width,
                                    y as f32 * self.tile_size.height,
                                );

                            let texture = if let Some(texture) = tile.texture {
                                texture
                            } else {
                                let tileset = self.tilesets.get(&tile.tileset_id).unwrap();

                                get_texture(&tileset.texture_id)
                            };

                            draw_texture(
                                world_position.x,
                                world_position.y,
                                texture,
                                DrawTextureParams {
                                    source: Some(Rect::new(
                                        tile.texture_coords.x, // + 0.1,
                                        tile.texture_coords.y, // + 0.1,
                                        self.tile_size.width,  // - 0.2,
                                        self.tile_size.height, // - 0.2,
                                    )),
                                    dest_size: Some(self.tile_size),
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
        Color::new(0.0, 0.0, 0.0, 1.0)
    }

    #[cfg(any(target_family = "unix", target_family = "windows"))]
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    #[cfg(target_family = "wasm")]
    pub fn save<P: AsRef<Path>>(&self, _: P) -> Result<()> {
        Ok(())
    }

    pub fn get_random_spawn_point(&self) -> Vec2 {
        let i = crate::rand::gen_range(0, self.spawn_points.len()) as usize;
        self.spawn_points[i]
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
        let next = if self.current.0 + 1 >= self.rect.x + self.rect.width {
            (self.rect.x, self.current.1 + 1)
        } else {
            (self.current.0 + 1, self.current.1)
        };

        let i = (self.current.1 * self.layer.grid_size.width + self.current.0) as usize;
        if self.current.1 >= self.rect.y + self.rect.height || i >= self.layer.tiles.len() {
            return None;
        }

        let res = Some((self.current.0, self.current.1, &self.layer.tiles[i]));

        self.current = next;

        res
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapLayerKind {
    TileLayer,
    ObjectLayer,
}

impl MapLayerKind {
    pub fn options() -> &'static [&'static str] {
        &["Tiles", "Objects"]
    }
}

#[cfg(feature = "macroquad-backend")]
impl ComboBoxValue for MapLayerKind {
    fn get_index(&self) -> usize {
        match self {
            Self::TileLayer => 0,
            Self::ObjectLayer => 1,
        }
    }

    fn set_index(&mut self, index: usize) {
        *self = match index {
            0 => Self::TileLayer,
            1 => Self::ObjectLayer,
            _ => unreachable!(),
        }
    }

    fn get_options(&self) -> Vec<String> {
        Self::options().iter().map(|s| s.to_string()).collect()
    }
}

impl Default for MapLayerKind {
    fn default() -> Self {
        MapLayerKind::TileLayer
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MapLayer {
    pub id: String,
    pub kind: MapLayerKind,
    #[serde(default)]
    pub has_collision: bool,
    pub grid_size: Size<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tiles: Vec<Option<MapTile>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub objects: Vec<MapObject>,
    #[serde(default)]
    pub is_visible: bool,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, MapProperty>,
}

impl MapLayer {
    pub fn new(id: &str, kind: MapLayerKind, has_collision: bool, grid_size: Size<u32>) -> Self {
        let has_collision = if kind == MapLayerKind::TileLayer {
            has_collision
        } else {
            false
        };

        let mut tiles = Vec::new();
        tiles.resize((grid_size.width * grid_size.height) as usize, None);

        MapLayer {
            id: id.to_string(),
            kind,
            has_collision,
            tiles,
            grid_size,
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
            grid_size: Size::zero(),
            tiles: Vec::new(),
            objects: Vec::new(),
            is_visible: true,
            properties: HashMap::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MapTile {
    pub tile_id: u32,
    pub tileset_id: String,
    pub texture_id: String,
    #[serde(skip)]
    pub texture: Option<Texture2D>,
    #[serde(with = "crate::parsing::vec2_def")]
    pub texture_coords: Vec2,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<String>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapObjectKind {
    Item,
    Environment,
    Decoration,
}

impl MapObjectKind {
    const ITEM: &'static str = "item";
    const ENVIRONMENT: &'static str = "environment";
    const DECORATION: &'static str = "decoration";

    pub fn options() -> &'static [&'static str] {
        &["Item", "Environment", "Decoration"]
    }
}

impl From<String> for MapObjectKind {
    fn from(str: String) -> Self {
        if str == Self::ITEM {
            Self::Item
        } else if str == Self::ENVIRONMENT {
            Self::Environment
        } else if str == Self::DECORATION {
            Self::Decoration
        } else {
            let str = if str.is_empty() {
                "NO_OBJECT_TYPE"
            } else {
                &str
            };

            unreachable!("Invalid MapObjectKind '{}'", str)
        }
    }
}

impl From<MapObjectKind> for String {
    fn from(kind: MapObjectKind) -> String {
        match kind {
            MapObjectKind::Item => MapObjectKind::ITEM.to_string(),
            MapObjectKind::Environment => MapObjectKind::ENVIRONMENT.to_string(),
            MapObjectKind::Decoration => MapObjectKind::DECORATION.to_string(),
        }
    }
}

#[cfg(feature = "macroquad-backend")]
impl ComboBoxValue for MapObjectKind {
    fn get_index(&self) -> usize {
        match self {
            Self::Item => 0,
            Self::Environment => 1,
            Self::Decoration => 2,
        }
    }

    fn set_index(&mut self, index: usize) {
        *self = match index {
            0 => Self::Item,
            1 => Self::Environment,
            2 => Self::Decoration,
            _ => unreachable!(),
        }
    }

    fn get_options(&self) -> Vec<String> {
        Self::options().iter().map(|s| s.to_string()).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapObject {
    pub id: String,
    pub kind: MapObjectKind,
    #[serde(with = "crate::parsing::vec2_def")]
    pub position: Vec2,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, MapProperty>,
}

impl MapObject {
    pub fn new(id: &str, kind: MapObjectKind, position: Vec2) -> Self {
        MapObject {
            id: id.to_string(),
            kind,
            position,
            properties: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapTileset {
    pub id: String,
    pub texture_id: String,
    pub texture_size: Size<u32>,
    pub tile_size: Size<f32>,
    pub grid_size: Size<u32>,
    pub first_tile_id: u32,
    pub tile_cnt: u32,
    #[serde(
        default = "MapTileset::default_tile_subdivisions",
        with = "crate::parsing::uvec2_def"
    )]
    pub tile_subdivisions: UVec2,
    pub autotile_mask: Vec<bool>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tile_attributes: HashMap<u32, Vec<String>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, MapProperty>,
    #[serde(skip)]
    pub bitmasks: Option<Vec<u32>>,
}

impl MapTileset {
    pub fn new(
        id: &str,
        texture_id: &str,
        texture_size: Size<u32>,
        tile_size: Size<f32>,
        first_tile_id: u32,
    ) -> Self {
        let grid_size = Size::new(
            texture_size.width / tile_size.width as u32,
            texture_size.height / tile_size.height as u32,
        );

        let tile_subdivisions = Self::default_tile_subdivisions();

        let subtile_grid_size = grid_size * tile_subdivisions;

        let subtile_cnt = (subtile_grid_size.width * subtile_grid_size.height) as usize;

        let mut autotile_mask = vec![];
        autotile_mask.resize(subtile_cnt, false);

        MapTileset {
            id: id.to_string(),
            texture_id: texture_id.to_string(),
            texture_size,
            tile_size,
            grid_size,
            first_tile_id,
            tile_cnt: grid_size.width * grid_size.height,
            tile_subdivisions,
            autotile_mask,
            tile_attributes: HashMap::new(),
            properties: HashMap::new(),
            bitmasks: None,
        }
    }

    pub fn get_texture_coords(&self, tile_id: u32) -> Vec2 {
        let x = (tile_id % self.grid_size.width) as f32 * self.tile_size.width;
        let y = (tile_id / self.grid_size.width) as f32 * self.tile_size.height;
        vec2(x, y)
    }

    pub fn default_tile_subdivisions() -> UVec2 {
        uvec2(3, 3)
    }

    pub fn get_bitmasks(&self) -> Option<Vec<u32>> {
        //Get autotile mask bitmasks
        let tsub_x = self.tile_subdivisions.x as usize;
        let tsub_y = self.tile_subdivisions.y as usize;
        let atmsk_width = self.grid_size.width as usize * tsub_x;

        let mut bitmasks_vec: Vec<Vec<bool>> =
            vec![vec![]; self.autotile_mask.len() / (tsub_x * tsub_y)];
        let mut bitmasks: Vec<u32> = vec![0; self.autotile_mask.len() / (tsub_x * tsub_y)];

        let mut trow_off = 0;
        for i in 0..self.autotile_mask.len() / atmsk_width {
            if i != 0 && i % tsub_y == 0 {
                trow_off += atmsk_width / tsub_x;
            }
            let row = self.autotile_mask[i * atmsk_width..i * atmsk_width + atmsk_width].to_vec();

            for x in 0..row.len() / tsub_x {
                let tile_row = row[x * tsub_x..x * tsub_x + tsub_x].to_vec();

                bitmasks_vec[x + trow_off].extend(tile_row);
            }
        }

        for (n, surrounding_tiles) in bitmasks_vec.iter().enumerate() {
            for (i, b) in surrounding_tiles.iter().enumerate() {
                if *b && i < 4 {
                    bitmasks[n] += 2_u32.pow(i as u32);
                } else if *b && i > 4 {
                    bitmasks[n] += 2_u32.pow(i as u32 - 1);
                }
            }
        }
        Some(bitmasks)
    }
}

pub fn draw_map(world: &mut World, _delta_time: f32) -> Result<()> {
    let camera_position = camera_position();

    for (_, map) in world.query_mut::<&Map>() {
        map.draw(None, camera_position);
    }

    Ok(())
}

static mut MAPS: Vec<MapResource> = Vec::new();

pub fn iter_maps() -> Iter<'static, MapResource> {
    unsafe { MAPS.iter() }
}

pub fn try_get_map(index: usize) -> Option<&'static MapResource> {
    unsafe { MAPS.get(index) }
}

pub fn get_map(index: usize) -> &'static MapResource {
    try_get_map(index).unwrap()
}

const MAP_RESOURCES_FILE: &str = "maps";

pub const MAP_EXPORTS_DEFAULT_DIR: &str = "maps";
pub const MAP_EXPORTS_EXTENSION: &str = "json";
pub const MAP_EXPORT_NAME_MIN_LEN: usize = 1;

pub const MAP_PREVIEW_PLACEHOLDER_PATH: &str = "maps/no_preview.png";
pub const MAP_PREVIEW_PLACEHOLDER_ID: &str = "map_preview_placeholder";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMetadata {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub path: String,
    pub preview_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_format: Option<TextureFormat>,
    #[serde(default, skip_serializing_if = "crate::parsing::is_false")]
    pub is_tiled_map: bool,
    #[serde(default, skip_serializing_if = "crate::parsing::is_false")]
    pub is_user_map: bool,
}

#[derive(Clone)]
pub struct MapResource {
    pub map: Map,
    pub preview: Texture2D,
    pub meta: MapMetadata,
}

pub fn create_map(
    name: &str,
    description: Option<&str>,
    tile_size: Vec2,
    grid_size: UVec2,
) -> Result<MapResource> {
    let description = description.map(|str| str.to_string());

    let map_path = Path::new(MAP_EXPORTS_DEFAULT_DIR)
        .join(map_name_to_filename(name))
        .with_extension(MAP_EXPORTS_EXTENSION);

    let preview_path = Path::new(MAP_PREVIEW_PLACEHOLDER_PATH);

    let meta = MapMetadata {
        name: name.to_string(),
        description,
        path: map_path.to_string_lossy().to_string(),
        preview_path: preview_path.to_string_lossy().to_string(),
        preview_format: None,
        is_tiled_map: false,
        is_user_map: true,
    };

    let map = Map::new(tile_size, grid_size);

    let preview = get_texture(MAP_PREVIEW_PLACEHOLDER_ID);

    Ok(MapResource { map, preview, meta })
}

pub fn save_map(map_resource: &MapResource) -> Result<()> {
    let assets_dir = assets_dir();
    let export_dir = Path::new(&assets_dir).join(&map_resource.meta.path);

    {
        let maps: &mut Vec<MapResource> = unsafe { MAPS.borrow_mut() };

        if export_dir.exists() {
            let mut i = 0;
            while i < maps.len() {
                let res = &maps[i];
                if res.meta.path == map_resource.meta.path {
                    if res.meta.is_user_map {
                        maps.remove(i);
                        break;
                    } else {
                        return Err(formaterr!(
                                ErrorKind::General,
                                "Resources: The path '{}' is in use and it is not possible to overwrite. Please choose a different map name",
                                &map_resource.meta.path,
                            ));
                    }
                }

                i += 1;
            }
        }

        map_resource.map.save(export_dir)?;

        maps.push(map_resource.clone());
    }

    save_maps_file()?;

    Ok(())
}

pub fn delete_map(index: usize) -> Result<()> {
    let map_resource = unsafe { MAPS.remove(index) };

    let assets_dir = assets_dir();
    let path = Path::new(&assets_dir).join(&map_resource.meta.path);

    fs::remove_file(path)?;

    save_maps_file()?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn save_maps_file() -> Result<()> {
    let assets_dir = assets_dir();
    let maps_file_path = Path::new(&assets_dir)
        .join(MAP_RESOURCES_FILE)
        .with_extension(DEFAULT_RESOURCE_FILE_EXTENSION);

    let metadata: Vec<MapMetadata> = iter_maps().map(|res| res.meta.clone()).collect();

    let str = serde_json::to_string_pretty(&metadata)?;
    fs::write(maps_file_path, &str)?;

    Ok(())
}

pub fn is_valid_map_export_path<P: AsRef<Path>>(path: P, should_overwrite: bool) -> bool {
    let path = path.as_ref();

    if let Some(file_name) = path.file_name() {
        if is_valid_map_file_name(&file_name.to_string_lossy().to_string()) {
            let res = iter_maps().find(|res| Path::new(&res.meta.path) == path);

            if let Some(res) = res {
                return res.meta.is_user_map && should_overwrite;
            }

            return true;
        }
    }

    false
}

pub fn map_name_to_filename(name: &str) -> String {
    name.replace(' ', "_").replace('.', "_").to_lowercase()
}

pub fn is_valid_map_file_name(file_name: &str) -> bool {
    if file_name.len() - MAP_EXPORTS_EXTENSION.len() > MAP_EXPORT_NAME_MIN_LEN {
        if let Some(extension) = Path::new(file_name).extension() {
            return extension == MAP_EXPORTS_EXTENSION;
        }
    }

    false
}

pub async fn load_maps<P: AsRef<Path>>(
    path: P,
    ext: &str,
    is_required: bool,
    should_overwrite: bool,
) -> Result<()> {
    let maps: &mut Vec<MapResource> = unsafe { MAPS.borrow_mut() };

    if should_overwrite {
        maps.clear();
    }

    let maps_file_path = path.as_ref().join(MAP_RESOURCES_FILE).with_extension(ext);

    match read_from_file(&maps_file_path).await {
        Err(err) => {
            if is_required {
                return Err(err.into());
            }
        }
        Ok(bytes) => {
            let metadata: Vec<MapMetadata> = deserialize_bytes_by_extension(ext, &bytes)?;

            for meta in metadata {
                let map_path = path.as_ref().join(&meta.path);
                let preview_path = path.as_ref().join(&meta.preview_path);

                let map = if meta.is_tiled_map {
                    Map::load_tiled(map_path, None).await?
                } else {
                    Map::load(map_path).await?
                };

                let preview = load_texture_file(
                    &preview_path,
                    meta.preview_format,
                    None,
                    TextureFilterMode::Nearest,
                    None,
                )
                .await?;

                let res = MapResource { map, preview, meta };

                maps.push(res)
            }
        }
    }

    Ok(())
}
