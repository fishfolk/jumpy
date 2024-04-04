use crate::prelude::*;

#[derive(Clone, Debug, HasSchema)]
#[schema(no_default)]
pub struct MapPool {
    pub maps: Vec<Handle<MapMeta>>,
    pub current_map: Handle<MapMeta>,
}

impl MapPool {
    /// Convert to [`MapPoolNetwork`] which is serializable for replication.
    pub fn into_network(&self, assets: &AssetServer) -> MapPoolNetwork {
        MapPoolNetwork {
            maps: self.maps.iter().map(|h| h.network_handle(assets)).collect(),
            current_map: self.current_map.network_handle(assets),
        }
    }

    /// Convert [`MapPoolNetwork`] into a [`MapPool`]
    pub fn from_network(map_pool: MapPoolNetwork, assets: &AssetServer) -> MapPool {
        MapPool {
            maps: map_pool
                .maps
                .iter()
                .map(|h| h.into_handle(assets))
                .collect(),
            current_map: map_pool.current_map.into_handle(assets),
        }
    }

    /// Make a `MapPool` consisting of single map.
    pub fn from_single_map(map: Handle<MapMeta>) -> Self {
        Self {
            maps: vec![map],
            current_map: map,
        }
    }

    /// Construct `MapPool` from slice of maps.
    /// Current map is first in array.
    pub fn from_slice(maps: &[Handle<MapMeta>]) -> Self {
        Self {
            maps: maps.into(),
            current_map: maps[0],
        }
    }

    /// Randomize current map. Updates `curent_map` on self and returns `Handle<MapMeta>`.
    pub fn randomize_current_map(&mut self, rng: &GlobalRng) -> Handle<MapMeta> {
        self.current_map = *rng.sample(&self.maps).unwrap();
        self.current_map
    }
}

#[derive(Serialize, Deserialize)]
pub struct MapPoolNetwork {
    pub maps: Vec<NetworkHandle<MapMeta>>,
    pub current_map: NetworkHandle<MapMeta>,
}
