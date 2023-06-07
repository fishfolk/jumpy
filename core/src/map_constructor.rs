//! Map constructor implementations.
//!
//! Map constructors are algorithms that can be used to create or edit game maps, usually through
//! the jumpy editor.

use crate::editor::MapManager;

pub mod shiftnanigans;

pub trait MapConstructor {
    fn construct_map(&self, map_manager: &mut MapManager);
}
