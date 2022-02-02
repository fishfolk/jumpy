use core::Result;

use crate::Map;

use super::UndoableAction;

pub struct EditorHistory {
    undo_stack: Vec<Box<dyn UndoableAction>>,
    redo_stack: Vec<Box<dyn UndoableAction>>,
}

impl EditorHistory {
    pub fn new() -> Self {
        EditorHistory {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn apply(&mut self, mut action: Box<dyn UndoableAction>, map: &mut Map) -> Result<()> {
        if !action.is_redundant(map) {
            action.apply(map)?;
            self.undo_stack.push(action);
            self.redo_stack.clear();
        }

        Ok(())
    }

    pub fn undo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(mut action) = self.undo_stack.pop() {
            action.undo(map)?;
            self.redo_stack.push(action);
        }

        Ok(())
    }

    pub fn redo(&mut self, map: &mut Map) -> Result<()> {
        if let Some(mut action) = self.redo_stack.pop() {
            action.redo(map)?;
            self.undo_stack.push(action);
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
