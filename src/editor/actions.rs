use std::result;

use crate::{
  map::{
      Map,
      MapLayer,
      MapLayerKind,
      CollisionKind,
  }
};

#[derive(Debug, Clone)]
pub enum EditorAction {
    Undo,
    Redo,
    SelectLayer(String),
    CreateLayer {
        id: String,
        kind: MapLayerKind,
        index: Option<usize>,
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

pub struct CreateLayer {
    id: String,
    kind: MapLayerKind,
    draw_order_index: Option<usize>,
}

impl CreateLayer {
    pub fn new(id: &str, kind: MapLayerKind, draw_order_index: Option<usize>) -> Self {
        CreateLayer {
            id: id.to_string(),
            kind,
            draw_order_index,
        }
    }
}

impl UndoableAction for CreateLayer {
    fn apply(&mut self, map: &mut Map) -> Result {
        let layer = MapLayer::new(&self.id, self.kind);
        map.layers.insert(self.id.clone(), layer);
        if let Some(i) = self.draw_order_index {
            if i > map.draw_order.len() - 1 {
                return Err("CreateLayer: The actions draw order index is out of bounds");
            }

            map.draw_order.insert(i, self.id.clone());
        } else {
            map.draw_order.push(self.id.clone());
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result {
        if map.layers.remove(&self.id).is_none() {
            return Err("CreateLayer (Undo): The layer does not exist");
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
