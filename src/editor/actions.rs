use std::{
    fmt,
    result,
};

use macroquad::{
    experimental::{
        collections::storage,
    },
    prelude::*,
};
use crate::{
    Resources,
    map::{
        Map,
        MapLayer,
        MapLayerKind,
        CollisionKind,
        MapTile,
        MapTileset,
    }
};

#[derive(Debug, Clone)]
pub enum EditorAction {
    Undo,
    Redo,
    SelectTile {
        id: u32,
        tileset_id: String,
    },
    OpenCreateLayerWindow,
    CloseCreateLayerWindow,
    SelectLayer(String),
    SetLayerDrawOrderIndex {
        id: String,
        index: usize,
    },
    CreateLayer {
        id: String,
        kind: MapLayerKind,
        draw_order_index: Option<usize>,
    },
    DeleteLayer(String),
    SelectTileset(String),
    OpenCreateTilesetWindow,
    CloseCreateTilesetWindow,
    CreateTileset {
        id: String,
        texture_id: String,
    },
    DeleteTileset(String),
    PlaceTile {
        id: u32,
        layer_id: String,
        tileset_id: String,
        coords: UVec2,
    },
    RemoveTile {
        layer_id: String,
        coords: UVec2,
    },
}

pub type Error = &'static str;

pub type Result = result::Result<(), Error>;

pub trait UndoableAction {
    fn apply(&mut self, _map: &mut Map) -> Result;

    fn undo(&mut self, _map: &mut Map) -> Result;

    fn redo(&mut self, map: &mut Map) -> Result {
        self.apply(map)
    }
}

#[derive(Debug)]
pub struct SetLayerDrawOrderIndex {
    id: String,
    draw_order_index: usize,
    old_draw_order_index: Option<usize>,
}

impl SetLayerDrawOrderIndex {
    pub fn new(id: String, draw_order_index: usize) -> Self {
        SetLayerDrawOrderIndex {
            id,
            draw_order_index,
            old_draw_order_index: None,
        }
    }
}

impl UndoableAction for SetLayerDrawOrderIndex {
    fn apply(&mut self, map: &mut Map) -> Result {
        for i in 0..map.draw_order.len() {
            let id = map.draw_order.get(i).unwrap();
            if id == &self.id {
                self.old_draw_order_index = Some(i);
                map.draw_order.remove(i);
                break;
            }
        }

        if self.old_draw_order_index.is_none() {
            return Err("SetLayerDrawOrderIndex: Could not find the specified layer in the map draw order array");
        }

        if self.draw_order_index >= map.draw_order.len() {
            map.draw_order.push(self.id.clone());
        } else {
            map.draw_order.insert(self.draw_order_index, self.id.clone());
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result {
        if map.draw_order.remove(self.draw_order_index) != self.id {
            return Err("SetLayerDrawOrderIndex (Undo): There was a mismatch between the layer id found in at the specified draw order index and the id stored in the action");
        }

        if let Some(i) = self.old_draw_order_index {
            if i > map.draw_order.len() {
                map.draw_order.push(self.id.clone());
            } else {
                map.draw_order.insert(i, self.id.clone());
            }
        } else {
            return Err("SetLayerDrawOrderIndex (Undo): No old draw order index was found")
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct CreateLayer {
    id: String,
    kind: MapLayerKind,
    draw_order_index: Option<usize>,
}

impl CreateLayer {
    pub fn new(id: String, kind: MapLayerKind, draw_order_index: Option<usize>) -> Self {
        CreateLayer {
            id,
            kind,
            draw_order_index,
        }
    }
}

impl UndoableAction for CreateLayer {
    fn apply(&mut self, map: &mut Map) -> Result {
        if map.layers.contains_key(&self.id) {
            let len = map.draw_order.len();
            for i in 0..len {
                let id = map.draw_order.get(i).unwrap();
                if id == &self.id {
                    map.draw_order.remove(i);
                    break;
                }
            }
        }

        let layer = MapLayer::new(&self.id, self.kind);
        map.layers.insert(self.id.clone(), layer);

        if let Some(i) = self.draw_order_index {
            if i <= map.draw_order.len() {
                map.draw_order.insert(i, self.id.clone());

                return Ok(());
            }
        }

        map.draw_order.push(self.id.clone());

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result {
        if map.layers.remove(&self.id).is_none() {
            return Err("CreateLayer (Undo): The specified layer does not exist");
        }

        if let Some(i) = self.draw_order_index {
            let id = map.draw_order.remove(i);
            if id != self.id {
                return Err("CreateLayer (Undo): There is a mismatch between the actions layer id and the one found at the actions draw order index");
            }
        } else {
            map.draw_order.retain(|id| id != &self.id);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DeleteLayer {
    id: String,
    layer: Option<MapLayer>,
    draw_order_index: Option<usize>,
}

impl DeleteLayer {
    pub fn new(id: String) -> Self {
        DeleteLayer {
            id,
            layer: None,
            draw_order_index: None,
        }
    }
}

impl UndoableAction for DeleteLayer {
    fn apply(&mut self, map: &mut Map) -> Result {
        if let Some(layer) = map.layers.remove(&self.id) {
            self.layer = Some(layer);
        } else {
            return Err("DeleteLayer: The specified layer does not exist");
        }

        let len = map.draw_order.len();
        for i in 0..len {
            let layer_id = map.draw_order.get(i).unwrap();
            if layer_id == &self.id {
                map.draw_order.remove(i);
                self.draw_order_index = Some(i);
                break;
            }
        }

        if self.draw_order_index.is_none() {
            return Err("DeleteLayer: The specified layer was not found in the draw order array");
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result {
        if let Some(layer) = self.layer.take() {
            map.layers.insert(self.id.clone(), layer);
            if let Some(i) = self.draw_order_index {
                if i >= map.draw_order.len() {
                    map.draw_order.push(self.id.clone());
                } else {
                    map.draw_order.insert(i, self.id.clone());
                }
            } else {
                return Err("DeleteLayer (Undo): No draw order index stored in action. Undo was probably called on an action that was never applied");
            }
        } else {
            return Err("DeleteLayer (Undo): No layer stored in action. Undo was probably called on an action that was never applied");
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct CreateTileset {
    id: String,
    texture_id: String,
}

impl CreateTileset {
    pub fn new(id: String, texture_id: String) -> Self {
        CreateTileset {
            id,
            texture_id,
        }
    }
}

impl UndoableAction for CreateTileset {
    fn apply(&mut self, map: &mut Map) -> Result {
        let resources = storage::get::<Resources>();
        if let Some(texture) = resources.textures.get(&self.texture_id) {
            let texture_size = uvec2(texture.width() as u32, texture.height() as u32);
            let mut first_tile_id = 1;
            for (_, tileset) in &map.tilesets {
                let next_tile_id = tileset.first_tile_id + tileset.tile_cnt;
                if next_tile_id > first_tile_id {
                    first_tile_id = next_tile_id;
                }
            }

            let tileset = MapTileset::new(
                &self.id,
                &self.texture_id,
                texture_size,
                map.tile_size,
                first_tile_id,
            );

            map.tilesets.insert(self.id.clone(), tileset);
        } else {
            return Err("CreateTileset: The specified texture does not exist");
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result {
        if map.tilesets.remove(&self.id).is_none() {
            return Err("CreateTileset (Undo): The specified tileset does not exist. Undo was probably called on an action that was never applied");
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DeleteTileset {
    id: String,
    tileset: Option<MapTileset>,
}

impl DeleteTileset {
    pub fn new(id: String) -> Self {
        DeleteTileset {
            id,
            tileset: None,
        }
    }
}

impl UndoableAction for DeleteTileset {
    fn apply(&mut self, map: &mut Map) -> Result {
        if let Some(tileset) = map.tilesets.remove(&self.id) {
            self.tileset = Some(tileset);
        } else {
            return Err("DeleteTileset: The specified tileset does not exist");
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result {
        if let Some(tileset) = self.tileset.take() {
            map.tilesets.insert(self.id.clone(), tileset);
        } else {
            return Err("DeleteTileset (Undo): No tileset stored in action. Undo was probably called on an action that was never applied");
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct PlaceTile {
    id: u32,
    layer_id: String,
    tileset_id: String,
    coords: UVec2,
    replaced_tile: Option<MapTile>,
}

impl PlaceTile {
    pub fn new(id: u32, layer_id: String, tileset_id: String, coords: UVec2) -> Self {
        PlaceTile {
            id,
            layer_id,
            tileset_id,
            coords,
            replaced_tile: None,
        }
    }
}

impl UndoableAction for PlaceTile {
    fn apply(&mut self, map: &mut Map) -> Result {
        if let Some(tileset) = map.tilesets.get(&self.tileset_id) {
            let texture_id = tileset.texture_id.clone();
            let texture_coords = tileset.get_texture_coords(self.id);

            let i = map.to_index(self.coords);

            if let Some(layer) = map.layers.get_mut(&self.layer_id) {
                if let MapLayerKind::TileLayer = layer.kind {
                    self.replaced_tile = layer.tiles.remove(i as usize);

                    let tile = MapTile {
                        tile_id: self.id,
                        tileset_id: self.tileset_id.clone(),
                        texture_id,
                        texture_coords,
                        attributes: vec!(),
                    };

                    layer.tiles.insert(i as usize, Some(tile));
                } else {
                    return Err("PlaceTile: The specified layer is not a tile layer");
                }
            } else {
                return Err("PlaceTile: The specified layer does not exist");
            }
        } else {
            return Err("PlaceTile: The specified tileset does not exist");
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result {
        let i = map.to_index(self.coords);

        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let MapLayerKind::TileLayer = layer.kind {
                if layer.tiles.remove(i as usize).is_none() {
                    return Err("PlaceTile (Undo): No tile at the specified coords, in the specified layer. Undo was probably called on an action that was never applied");
                }

                if let Some(tile) = self.replaced_tile.take() {
                    layer.tiles.insert(i as usize, Some(tile));
                } else {
                    layer.tiles.insert(i as usize, None);
                }
            } else {
                return Err("PlaceTile (Undo): The specified layer is not a tile layer");
            }
        } else {
            return Err("PlaceTile (Undo): The specified layer does not exist");
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct RemoveTile {
    layer_id: String,
    coords: UVec2,
    tile: Option<MapTile>,
}

impl RemoveTile {
    pub fn new(layer_id: String, coords: UVec2) -> Self {
        RemoveTile {
            layer_id,
            coords,
            tile: None,
        }
    }
}

impl UndoableAction for RemoveTile {
    fn apply(&mut self, map: &mut Map) -> Result {
        let i = map.to_index(self.coords);

        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let MapLayerKind::TileLayer = layer.kind {
                if let Some(tile) = layer.tiles.remove(i as usize) {
                    self.tile = Some(tile);

                    layer.tiles.insert(i, None);
                } else {
                    return Err("RemoveTile: No tile at the specified coords, in the specified layer. Undo was probably called on an action that was never applied");
                }
            } else {
                return Err("RemoveTile: The specified layer is not a tile layer");
            }
        } else {
            return Err("RemoveTile: The specified layer does not exist");
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result {
        let i = map.to_index(self.coords);

        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let MapLayerKind::TileLayer = layer.kind {
                if let Some(tile) = self.tile.take() {
                    layer.tiles.insert(i, Some(tile));
                } else {
                    return Err("RemoveTile (Undo): No tile stored in action. Undo was probably called on an action that was never applied");
                }
            } else {
                return Err("RemoveTile (Undo): The specified layer is not a tile layer");
            }
        } else {
            return Err("RemoveTile (Undo): The specified layer does not exist");
        }

        Ok(())
    }
}