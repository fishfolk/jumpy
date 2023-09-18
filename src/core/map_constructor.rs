//! Map constructor implementations.
//!
//! Map constructors are algorithms that can be used to create or edit game maps, usually through
//! the jumpy editor.

use super::editor::MapManager;

pub mod shiftnanigans;

/// Trait implemented by map constructors.
pub trait MapConstructor {
    /// Take the map manager and use it to either modify or construct a new map.
    fn construct_map(&self, map_manager: &mut MapManager);
}
