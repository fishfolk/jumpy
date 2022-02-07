use std::any::TypeId;
use std::cmp::Ordering;

use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use core::error::{Error, ErrorKind, Result};

use crate::editor::gui::windows::Window;
use crate::map::{MapBackgroundLayer, MapObject, MapObjectKind};
use crate::{
    map::{Map, MapLayer, MapLayerKind, MapTile, MapTileset},
    Resources,
};

/// These are all the actions available for the GUI and other sub-systems of the editor.
/// If you need to perform multiple actions in one call, use the `Batch` variant.
#[derive(Debug, Clone)]
pub enum EditorAction {
    Batch(Vec<EditorAction>),
    Undo,
    Redo,
    SelectTool(Option<TypeId>),
    OpenBackgroundPropertiesWindow,
    UpdateBackground {
        color: Color,
        layers: Vec<MapBackgroundLayer>,
    },
    OpenCreateLayerWindow,
    OpenCreateTilesetWindow,
    OpenTilesetPropertiesWindow(String),
    OpenCreateObjectWindow {
        position: Vec2,
        layer_id: String,
    },
    OpenObjectPropertiesWindow {
        layer_id: String,
        index: usize,
    },
    OpenTilePropertiesWindow {
        layer_id: String,
        index: usize,
    },
    CloseWindow(TypeId),
    SelectTile {
        id: u32,
        tileset_id: String,
    },
    UpdateTileAttributes {
        index: usize,
        layer_id: String,
        attributes: Vec<String>,
    },
    SelectLayer(String),
    SetLayerDrawOrderIndex {
        id: String,
        index: usize,
    },
    CreateLayer {
        id: String,
        kind: MapLayerKind,
        has_collision: bool,
        index: Option<usize>,
    },
    DeleteLayer(String),
    UpdateLayer {
        id: String,
        is_visible: bool,
    },
    SelectTileset(String),
    OpenImportWindow(usize),
    Import {
        tilesets: Vec<MapTileset>,
        background_color: Option<Color>,
        background_layers: Vec<MapBackgroundLayer>,
    },
    CreateTileset {
        id: String,
        texture_id: String,
    },
    DeleteTileset(String),
    UpdateTileset {
        id: String,
        texture_id: String,
        autotile_mask: Vec<bool>,
    },
    SelectObject {
        index: usize,
        layer_id: String,
    },
    CreateObject {
        id: String,
        kind: MapObjectKind,
        position: Vec2,
        layer_id: String,
    },
    DeleteObject {
        index: usize,
        layer_id: String,
    },
    UpdateObject {
        layer_id: String,
        index: usize,
        id: String,
        kind: MapObjectKind,
        position: Vec2,
    },
    CreateSpawnPoint(Vec2),
    DeleteSpawnPoint(usize),
    MoveSpawnPoint {
        index: usize,
        position: Vec2,
    },
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
    CreateMap {
        name: String,
        description: Option<String>,
        tile_size: Vec2,
        grid_size: UVec2,
    },
    OpenCreateMapWindow,
    OpenMap(usize),
    OpenLoadMapWindow,
    SaveMap(Option<String>),
    OpenSaveMapWindow,
    DeleteMap(usize),
    ExitToMainMenu,
    QuitToDesktop,
}

impl EditorAction {
    pub fn batch(actions: &[EditorAction]) -> Self {
        Self::Batch(actions.to_vec())
    }

    pub fn close_window<T: Window + ?Sized + 'static>() -> Self {
        let id = TypeId::of::<T>();
        EditorAction::CloseWindow(id)
    }

    pub fn then(self, action: EditorAction) -> Self {
        Self::batch(&[self, action])
    }
}

/// All actions that modify map data should implement this trait
pub trait UndoableAction {
    fn apply(&mut self, _map: &mut Map) -> Result<()>;

    fn undo(&mut self, _map: &mut Map) -> Result<()>;

    fn redo(&mut self, map: &mut Map) -> Result<()> {
        self.apply(map)
    }

    /// Implement this for actions that can be redundant (ie. no change will take place if it is applied).
    /// This is to avoid history being filled up with repeat actions if user is holding input down
    /// for a long time, for example.
    /// This should not be used to circumvent bugs and errors, however. It is meant to stop the same
    /// action from firing several times in a row, for example from holding mouse down on the map
    /// (placing multiple tiles, with the same id, to the same coords).
    /// Edge cases, like an action wanting to delete a layer that does not exist, should be handled
    /// with errors, in stead (basically, things like that shouldn't happen, as it should be prevented
    /// at a higher level)
    fn is_redundant(&self, _map: &Map) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct UpdateBackgroundAction {
    color: Color,
    old_color: Option<Color>,
    layers: Vec<MapBackgroundLayer>,
    old_layers: Option<Vec<MapBackgroundLayer>>,
}

impl UpdateBackgroundAction {
    pub fn new(color: Color, layers: Vec<MapBackgroundLayer>) -> Self {
        UpdateBackgroundAction {
            color,
            old_color: None,
            layers,
            old_layers: None,
        }
    }
}

impl UndoableAction for UpdateBackgroundAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        self.old_color = Some(map.background_color);

        map.background_color = self.color;

        self.old_layers = Some(map.background_layers.clone());

        map.background_layers = self.layers.clone();

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(color) = self.old_color.take() {
            map.background_color = color;
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateBackgroundPropertiesAction (Undo): No old background color was found. Undo was probably called on an action that was never applied"));
        }

        if let Some(layers) = self.old_layers.take() {
            map.background_layers = layers;
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateBackgroundPropertiesAction (Undo): No old background layers was found. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct SetLayerDrawOrderIndexAction {
    id: String,
    draw_order_index: usize,
    old_draw_order_index: Option<usize>,
}

impl SetLayerDrawOrderIndexAction {
    pub fn new(id: String, draw_order_index: usize) -> Self {
        SetLayerDrawOrderIndexAction {
            id,
            draw_order_index,
            old_draw_order_index: None,
        }
    }
}

impl UndoableAction for SetLayerDrawOrderIndexAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        for i in 0..map.draw_order.len() {
            let id = map.draw_order.get(i).unwrap();
            if id == &self.id {
                self.old_draw_order_index = Some(i);
                map.draw_order.remove(i);
                break;
            }
        }

        if self.old_draw_order_index.is_none() {
            return Err(Error::new_const(ErrorKind::EditorAction, &"SetLayerDrawOrderIndexAction: Could not find the specified layer in the map draw order array"));
        }

        if self.draw_order_index > map.draw_order.len() {
            map.draw_order.push(self.id.clone());
        } else {
            map.draw_order
                .insert(self.draw_order_index, self.id.clone());
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if map.draw_order.remove(self.draw_order_index) != self.id {
            return Err(Error::new_const(ErrorKind::EditorAction, &"SetLayerDrawOrderIndexAction (Undo): There was a mismatch between the layer id found in at the specified draw order index and the id stored in the action"));
        }

        if let Some(i) = self.old_draw_order_index {
            if i > map.draw_order.len() {
                map.draw_order.push(self.id.clone());
            } else {
                map.draw_order.insert(i, self.id.clone());
            }
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"SetLayerDrawOrderIndexAction (Undo): No old draw order index was found. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }

    fn is_redundant(&self, map: &Map) -> bool {
        for i in 0..map.draw_order.len() {
            let id = map.draw_order.get(i).unwrap();
            if id == &self.id && i == self.draw_order_index {
                return true;
            }
        }

        false
    }
}

#[derive(Debug)]
pub struct UpdateTileAttributesAction {
    index: usize,
    layer_id: String,
    attributes: Vec<String>,
    old_attributes: Option<Vec<String>>,
}

impl UpdateTileAttributesAction {
    pub fn new(index: usize, layer_id: String, attributes: Vec<String>) -> Self {
        UpdateTileAttributesAction {
            index,
            layer_id,
            attributes,
            old_attributes: None,
        }
    }
}

impl UndoableAction for UpdateTileAttributesAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let Some(Some(tile)) = layer.tiles.get_mut(self.index) {
                self.old_attributes = Some(tile.attributes.clone());
                tile.attributes = self.attributes.clone();
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"UpdateTileAttributesAction: The specified tile does not exist",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateTileAttributesAction: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let Some(Some(tile)) = layer.tiles.get_mut(self.index) {
                if let Some(old_attributes) = self.old_attributes.take() {
                    tile.attributes = old_attributes;
                } else {
                    return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateTileAttributesAction (Undo): No old attributes stored in action. Undo was probably called on an action that was never applied"));
                }
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"UpdateTileAttributesAction (Undo): The specified tile does not exist",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateTileAttributesAction (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn is_redundant(&self, map: &Map) -> bool {
        if let Some(layer) = map.layers.get(&self.layer_id) {
            if let Some(Some(tile)) = layer.tiles.get(self.index) {
                let matching = tile
                    .attributes
                    .iter()
                    .zip(&self.attributes)
                    .filter(|&(a, b)| a == b)
                    .count();
                return matching == self.attributes.len();
            }
        }

        false
    }
}

#[derive(Debug)]
pub struct CreateLayerAction {
    id: String,
    kind: MapLayerKind,
    has_collision: bool,
    index: Option<usize>,
}

impl CreateLayerAction {
    pub fn new(id: String, kind: MapLayerKind, has_collision: bool, index: Option<usize>) -> Self {
        CreateLayerAction {
            id,
            kind,
            has_collision,
            index,
        }
    }
}

impl UndoableAction for CreateLayerAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
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

        let layer = MapLayer::new(&self.id, self.kind, self.has_collision, map.grid_size);
        map.layers.insert(self.id.clone(), layer);

        if let Some(i) = self.index {
            if i <= map.draw_order.len() {
                map.draw_order.insert(i, self.id.clone());

                return Ok(());
            }
        }

        map.draw_order.push(self.id.clone());

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if map.layers.remove(&self.id).is_none() {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"CreateLayerAction (Undo): The specified layer does not exist",
            ));
        }

        if let Some(i) = self.index {
            let id = map.draw_order.remove(i);
            if id != self.id {
                return Err(Error::new_const(ErrorKind::EditorAction, &"CreateLayerAction (Undo): There is a mismatch between the actions layer id and the one found at the actions draw order index"));
            }
        } else {
            map.draw_order.retain(|id| id != &self.id);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DeleteLayerAction {
    id: String,
    layer: Option<MapLayer>,
    draw_order_index: Option<usize>,
}

impl DeleteLayerAction {
    pub fn new(id: String) -> Self {
        DeleteLayerAction {
            id,
            layer: None,
            draw_order_index: None,
        }
    }
}

impl UndoableAction for DeleteLayerAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.remove(&self.id) {
            self.layer = Some(layer);
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"DeleteLayerAction: The specified layer does not exist",
            ));
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
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"DeleteLayerAction: The specified layer was not found in the draw order array",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = self.layer.take() {
            map.layers.insert(self.id.clone(), layer);
            if let Some(i) = self.draw_order_index {
                if i >= map.draw_order.len() {
                    map.draw_order.push(self.id.clone());
                } else {
                    map.draw_order.insert(i, self.id.clone());
                }
            } else {
                return Err(Error::new_const(ErrorKind::EditorAction, &"DeleteLayerAction (Undo): No draw order index stored in action. Undo was probably called on an action that was never applied"));
            }
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"DeleteLayerAction (Undo): No layer stored in action. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateLayerAction {
    id: String,
    is_visible: bool,
    old_is_visible: Option<bool>,
}

impl UpdateLayerAction {
    pub fn new(id: String, is_visible: bool) -> Self {
        UpdateLayerAction {
            id,
            is_visible,
            old_is_visible: None,
        }
    }
}

impl UndoableAction for UpdateLayerAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.id) {
            self.old_is_visible = Some(layer.is_visible);
            layer.is_visible = self.is_visible;
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateLayerAction: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.id) {
            if let Some(old_is_visible) = self.old_is_visible.take() {
                layer.is_visible = old_is_visible;
            } else {
                return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateLayerAction (Undo): No `old_is_visible` on action. Undo was probably called on an action that was never applied"));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateLayerAction (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct ImportAction {
    tilesets: Vec<MapTileset>,
    background_color: Option<Color>,
    old_background_color: Option<Color>,
    background_layers: Vec<MapBackgroundLayer>,
    old_background_layers: Vec<MapBackgroundLayer>,
}

impl ImportAction {
    pub fn new(
        tilesets: Vec<MapTileset>,
        background_color: Option<Color>,
        background_layers: Vec<MapBackgroundLayer>,
    ) -> Self {
        ImportAction {
            tilesets,
            background_color,
            old_background_color: None,
            background_layers,
            old_background_layers: Vec::new(),
        }
    }
}

impl UndoableAction for ImportAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        for tileset in &self.tilesets {
            let mut first_tile_id = 1;
            for tileset in map.tilesets.values() {
                let next_tile_id = tileset.first_tile_id + tileset.tile_cnt;
                if next_tile_id > first_tile_id {
                    first_tile_id = next_tile_id;
                }
            }

            let tileset = MapTileset {
                id: tileset.id.clone(),
                texture_id: tileset.texture_id.clone(),
                texture_size: tileset.texture_size,
                tile_size: tileset.tile_size,
                grid_size: tileset.grid_size,
                first_tile_id,
                tile_cnt: tileset.tile_cnt,
                tile_subdivisions: tileset.tile_subdivisions,
                autotile_mask: tileset.autotile_mask.clone(),
                tile_attributes: tileset.tile_attributes.clone(),
                properties: tileset.properties.clone(),
            };

            map.tilesets.insert(tileset.id.clone(), tileset);
        }

        if let Some(background_color) = self.background_color {
            self.old_background_color = Some(map.background_color);
            map.background_color = background_color;
        }

        self.old_background_layers = map.background_layers.clone();

        map.background_layers
            .append(&mut self.background_layers.clone());

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        for tileset in &self.tilesets {
            if map.tilesets.remove(&tileset.id).is_none() {
                return Err(Error::new_const(ErrorKind::EditorAction, &"ImportTilesetsAction (Undo): One of the imported tilesets could not be found in the map"));
            }
        }

        if let Some(background_color) = self.old_background_color.take() {
            map.background_color = background_color;
        }

        map.background_layers = self.old_background_layers.drain(..).collect();

        Ok(())
    }
}

#[derive(Debug)]
pub struct CreateTilesetAction {
    id: String,
    texture_id: String,
}

impl CreateTilesetAction {
    pub fn new(id: String, texture_id: String) -> Self {
        CreateTilesetAction { id, texture_id }
    }
}

impl UndoableAction for CreateTilesetAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        let resources = storage::get::<Resources>();
        if let Some(texture_entry) = resources.textures.get(&self.texture_id).cloned() {
            let texture_size = uvec2(
                texture_entry.texture.width() as u32,
                texture_entry.texture.height() as u32,
            );

            let mut first_tile_id = 1;
            for tileset in map.tilesets.values() {
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
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"CreateTilesetAction: The specified texture does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if map.tilesets.remove(&self.id).is_none() {
            return Err(Error::new_const(ErrorKind::EditorAction, &"CreateTilesetAction (Undo): The specified tileset does not exist. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DeleteTilesetAction {
    id: String,
    tileset: Option<MapTileset>,
}

impl DeleteTilesetAction {
    pub fn new(id: String) -> Self {
        DeleteTilesetAction { id, tileset: None }
    }
}

impl UndoableAction for DeleteTilesetAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        if let Some(tileset) = map.tilesets.remove(&self.id) {
            self.tileset = Some(tileset);
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"DeleteTilesetAction: The specified tileset does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(tileset) = self.tileset.take() {
            map.tilesets.insert(self.id.clone(), tileset);
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"DeleteTilesetAction (Undo): No tileset stored in action. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateTilesetAction {
    id: String,
    texture_id: String,
    old_texture_id: Option<String>,
    autotile_mask: Vec<bool>,
    old_autotile_mask: Option<Vec<bool>>,
}

impl UpdateTilesetAction {
    pub fn new(id: String, texture_id: String, autotile_mask: Vec<bool>) -> Self {
        UpdateTilesetAction {
            id,
            texture_id,
            old_texture_id: None,
            autotile_mask,
            old_autotile_mask: None,
        }
    }
}

impl UndoableAction for UpdateTilesetAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        if let Some(tileset) = map.tilesets.get_mut(&self.id) {
            self.old_texture_id = Some(tileset.texture_id.clone());
            tileset.texture_id = self.texture_id.clone();

            if self.autotile_mask.len() != tileset.autotile_mask.len() {
                return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateTilesetAction: There is a size mismatch between the actions autotile mask vector and the one on the tileset"));
            }

            self.old_autotile_mask = Some(tileset.autotile_mask.clone());
            tileset.autotile_mask = self.autotile_mask.clone();
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateTilesetAction: The specified tileset does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(tileset) = map.tilesets.get_mut(&self.id) {
            if let Some(old_texture_id) = &self.old_texture_id {
                tileset.texture_id = old_texture_id.clone();
            } else {
                return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateTilesetAction (Undo): No old texture id stored in action. Undo was probably called on an action that was never applied"));
            }

            if let Some(old_autotile_mask) = &self.old_autotile_mask {
                if old_autotile_mask.len() != tileset.autotile_mask.len() {
                    return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateTilesetAction (Undo): There is a size mismatch between the actions autotile mask vector and the one on the tileset"));
                }

                tileset.autotile_mask = old_autotile_mask.clone();
                self.old_autotile_mask = None;
            } else {
                return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateTilesetAction (Undo): No old autotile mask stored in action. Undo was probably called on an action that was never applied"));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateTilesetAction (Undo): The specified tileset does not exist",
            ));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct CreateObjectAction {
    id: String,
    kind: MapObjectKind,
    position: Vec2,
    layer_id: String,
}

impl CreateObjectAction {
    pub fn new(id: String, kind: MapObjectKind, position: Vec2, layer_id: String) -> Self {
        CreateObjectAction {
            id,
            kind,
            position,
            layer_id,
        }
    }
}

impl UndoableAction for CreateObjectAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            let object = MapObject::new(&self.id, self.kind, self.position);

            layer.objects.insert(0, object);
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"CreateObjectAction: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            layer.objects.remove(0);
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"CreateObjectAction (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DeleteObjectAction {
    index: usize,
    layer_id: String,
    object: Option<MapObject>,
}

impl DeleteObjectAction {
    pub fn new(index: usize, layer_id: String) -> Self {
        DeleteObjectAction {
            index,
            layer_id,
            object: None,
        }
    }
}

impl UndoableAction for DeleteObjectAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            let object = layer.objects.remove(self.index);
            self.object = Some(object);
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"DeleteObjectAction: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(object) = self.object.take() {
            if let Some(layer) = map.layers.get_mut(&self.layer_id) {
                layer.objects.insert(self.index, object);
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"DeleteObjectAction (Undo): The specified layer does not exist",
                ));
            }
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"DeleteObjectAction (Undo): No object stored in action. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateObjectAction {
    layer_id: String,
    index: usize,
    id: String,
    kind: MapObjectKind,
    position: Vec2,
    object: Option<MapObject>,
}

impl UpdateObjectAction {
    pub fn new(
        layer_id: String,
        index: usize,
        id: String,
        kind: MapObjectKind,
        position: Vec2,
    ) -> Self {
        UpdateObjectAction {
            layer_id,
            index,
            id,
            kind,
            position,
            object: None,
        }
    }
}

impl UndoableAction for UpdateObjectAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let Some(object) = layer.objects.get_mut(self.index) {
                self.object = Some(object.clone());

                object.id = self.id.clone();
                object.kind = self.kind;
                object.position = self.position;
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"UpdateObjectAction: The specified object index does not exist",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateObjectAction: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let Some(object) = self.object.take() {
                layer.objects[self.index] = object;
            } else {
                return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateObjectAction: No object found on action. Undo was probably called on an action that was never applied"));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateObjectAction (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct CreateSpawnPointAction {
    position: Vec2,
}

impl CreateSpawnPointAction {
    pub fn new(position: Vec2) -> Self {
        CreateSpawnPointAction { position }
    }
}

impl UndoableAction for CreateSpawnPointAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        map.spawn_points.push(self.position);

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        map.spawn_points.pop();

        Ok(())
    }
}

#[derive(Debug)]
pub struct DeleteSpawnPointAction {
    index: usize,
    spawn_point: Option<Vec2>,
}

impl DeleteSpawnPointAction {
    pub fn new(index: usize) -> Self {
        DeleteSpawnPointAction {
            index,
            spawn_point: None,
        }
    }
}

impl UndoableAction for DeleteSpawnPointAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        let spawn_point = map.spawn_points.remove(self.index);
        self.spawn_point = Some(spawn_point);

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(spawn_point) = self.spawn_point.take() {
            if self.index >= map.spawn_points.len() {
                map.spawn_points.push(spawn_point);
            } else {
                map.spawn_points.insert(self.index, spawn_point);
            }
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"DeleteSpawnPointAction (Undo): No spawn point saved in action. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct MoveSpawnPointAction {
    index: usize,
    position: Vec2,
    old_position: Option<Vec2>,
}

impl MoveSpawnPointAction {
    pub fn new(index: usize, position: Vec2) -> Self {
        MoveSpawnPointAction {
            index,
            position,
            old_position: None,
        }
    }
}

impl UndoableAction for MoveSpawnPointAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        let old_position = map.spawn_points.remove(self.index);
        self.old_position = Some(old_position);

        match map.spawn_points.len().cmp(&self.index) {
            Ordering::Equal => map.spawn_points.push(self.position),
            Ordering::Greater => map.spawn_points.insert(self.index, self.position),
            _ => {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"MoveSpawnPointAction: Index out of bounds",
                ))
            }
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(old_position) = self.old_position {
            map.spawn_points[self.index] = old_position;
        } else {
            return Err(Error::new_const(ErrorKind::EditorAction, &"MoveSpawnPointAction (Undo): No old position saved in action. Undo was probably called on an action that was never applied"));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct PlaceTileAction {
    id: u32,
    layer_id: String,
    tileset_id: String,
    coords: UVec2,
    replaced_tile: Option<MapTile>,
}

impl PlaceTileAction {
    pub fn new(id: u32, layer_id: String, tileset_id: String, coords: UVec2) -> Self {
        PlaceTileAction {
            id,
            layer_id,
            tileset_id,
            coords,
            replaced_tile: None,
        }
    }
}

impl UndoableAction for PlaceTileAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
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
                        attributes: vec![],
                    };

                    layer.tiles.insert(i as usize, Some(tile));
                } else {
                    return Err(Error::new_const(
                        ErrorKind::EditorAction,
                        &"PlaceTileAction: The specified layer is not a tile layer",
                    ));
                }
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"PlaceTileAction: The specified layer does not exist",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"PlaceTileAction: The specified tileset does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        let i = map.to_index(self.coords);

        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let MapLayerKind::TileLayer = layer.kind {
                if layer.tiles.get(i as usize).is_none() {
                    return Err(Error::new_const(ErrorKind::EditorAction, &"PlaceTileAction (Undo): No tile at the specified coords, in the specified layer. Undo was probably called on an action that was never applied"));
                }

                if let Some(tile) = self.replaced_tile.take() {
                    layer.tiles[i as usize] = Some(tile);
                } else {
                    layer.tiles[i as usize] = None;
                }
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"PlaceTileAction (Undo): The specified layer is not a tile layer",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"PlaceTileAction (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn is_redundant(&self, map: &Map) -> bool {
        if let Some(layer) = map.layers.get(&self.layer_id) {
            let i = map.to_index(self.coords);
            if let Some(Some(tile)) = layer.tiles.get(i) {
                return tile.tileset_id == self.tileset_id && tile.tile_id == self.id;
            }
        }

        false
    }
}

#[derive(Debug)]
pub struct RemoveTileAction {
    layer_id: String,
    coords: UVec2,
    tile: Option<MapTile>,
}

impl RemoveTileAction {
    pub fn new(layer_id: String, coords: UVec2) -> Self {
        RemoveTileAction {
            layer_id,
            coords,
            tile: None,
        }
    }
}

impl UndoableAction for RemoveTileAction {
    fn apply(&mut self, map: &mut Map) -> Result<()> {
        let i = map.to_index(self.coords);

        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let MapLayerKind::TileLayer = layer.kind {
                if let Some(tile) = layer.tiles.remove(i as usize) {
                    self.tile = Some(tile);

                    layer.tiles.insert(i, None);
                } else {
                    return Err(Error::new_const(ErrorKind::EditorAction, &"RemoveTileAction: No tile at the specified coords, in the specified layer. Undo was probably called on an action that was never applied"));
                }
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"RemoveTileAction: The specified layer is not a tile layer",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"RemoveTileAction: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        let i = map.to_index(self.coords);

        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let MapLayerKind::TileLayer = layer.kind {
                if let Some(old_tile) = self.tile.take() {
                    if let Some(tile) = layer.tiles.get_mut(i) {
                        *tile = Some(old_tile);
                    } else {
                        return Err(Error::new_const(ErrorKind::EditorAction, &"RemoveTileAction (Undo): No tile found vec entry in map at the index stored in action (this should not be possible, as the entry should be a `None` if the tile was empty)"));
                    }
                } else {
                    return Err(Error::new_const(ErrorKind::EditorAction, &"RemoveTileAction (Undo): No tile stored in action. Undo was probably called on an action that was never applied"));
                }
            } else {
                return Err(Error::new_const(
                    ErrorKind::EditorAction,
                    &"RemoveTileAction (Undo): The specified layer is not a tile layer",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"RemoveTileAction (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn is_redundant(&self, map: &Map) -> bool {
        let i = map.to_index(self.coords);
        if let Some(layer) = map.layers.get(&self.layer_id) {
            if let Some(tile) = layer.tiles.get(i) {
                return tile.is_none();
            }
        }

        false
    }
}
