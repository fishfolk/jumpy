use bones_lib::prelude::{CompMut, Transform};

use crate::prelude::{LoadedMap, SpawnedMapLayerMeta};

pub mod shiftnanigans;

trait MapConstructor {
    fn construct_map(&self, &mut spawned_map_layer_metas: CompMut<SpawnedMapLayerMeta>, &mut transforms: CompMut<Transform>) -> LoadedMap;
}

