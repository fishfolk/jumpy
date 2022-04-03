use core::error::ErrorKind;

use core::error::Error;

use core::error::Result;

use macroquad::prelude::Vec2;

use crate::map::Map;

use super::UndoableAction;

use crate::map::MapObject;

use crate::map::MapObjectKind;

#[derive(Debug)]
pub struct UpdateObject {
    layer_id: String,
    index: usize,
    id: String,
    kind: MapObjectKind,
    position: Vec2,
    object: Option<MapObject>,
}

impl UpdateObject {
    pub fn new(
        layer_id: String,
        index: usize,
        id: String,
        kind: MapObjectKind,
        position: Vec2,
    ) -> Self {
        UpdateObject {
            layer_id,
            index,
            id,
            kind,
            position,
            object: None,
        }
    }
}

impl UndoableAction for UpdateObject {
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
                    &"UpdateObject: The specified object index does not exist",
                ));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateObject: The specified layer does not exist",
            ));
        }

        Ok(())
    }

    fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(layer) = map.layers.get_mut(&self.layer_id) {
            if let Some(object) = self.object.take() {
                layer.objects[self.index] = object;
            } else {
                return Err(Error::new_const(ErrorKind::EditorAction, &"UpdateObject: No object found on action. Undo was probably called on an action that was never applied"));
            }
        } else {
            return Err(Error::new_const(
                ErrorKind::EditorAction,
                &"UpdateObject (Undo): The specified layer does not exist",
            ));
        }

        Ok(())
    }
}
