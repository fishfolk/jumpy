use crate::editor::MapManager;

pub mod shiftnanigans;

pub trait MapConstructor {
    fn construct_map(&self, map_manager: &mut MapManager);
}
