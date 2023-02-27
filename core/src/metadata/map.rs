use super::*;

#[derive(BonesBevyAsset, Serialize, Deserialize, Clone, TypeUlid, Debug, Default)]
#[ulid = "01GP264BT87MAAHMEK52Y5P7BW"]
#[asset_id = "map"]
#[serde(deny_unknown_fields)]
pub struct MapMeta {
    pub name: String,
    /// The parallax background layers
    #[serde(default)]
    pub background: BackgroundMeta,
    /// The background color of the map, behind the parallax layers
    pub background_color: ColorMeta,
    /// Size of the map in tiles
    pub grid_size: UVec2,
    /// The size of the tiles in pixels
    pub tile_size: Vec2,
    /// The layers of the map
    pub layers: Vec<MapLayerMeta>,
}

#[derive(BonesBevyAssetLoad, Serialize, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct BackgroundMeta {
    pub speed: Vec2,
    pub layers: Vec<ParallaxLayerMeta>,
}

#[derive(BonesBevyAssetLoad, Serialize, Deserialize, Clone, Debug, TypeUlid)]
#[serde(deny_unknown_fields)]
#[ulid = "01GPP1QJFVQN3HYW4N7ZE3S89Y"]
pub struct ParallaxLayerMeta {
    pub image: Handle<Image>,
    pub size: Vec2,
    pub depth: f32,
    pub scale: f32,
    #[serde(default)]
    pub offset: Vec2,
}

#[derive(BonesBevyAssetLoad, Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MapLayerMeta {
    pub id: String,
    #[serde(default)]
    pub tilemap: Option<Handle<Atlas>>,
    #[serde(default)]
    pub tiles: Vec<MapTileMeta>,
    #[serde(default)]
    pub elements: Vec<ElementSpawn>,
}

#[derive(BonesBevyAssetLoad, Serialize, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct ElementSpawn {
    pub pos: Vec2,
    pub element: Handle<ElementMeta>,
}

#[derive(BonesBevyAssetLoad, Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MapTileMeta {
    pub pos: UVec2,
    pub idx: u32,
    #[serde(default)]
    pub jump_through: bool,
}

impl MapMeta {
    /// Checks if the given position is out of the bounds of the map.
    pub fn is_out_of_bounds(&self, pos: &Vec3) -> bool {
        const KILL_ZONE_BORDER: f32 = 500.0;
        let map_width = self.grid_size.x as f32 * self.tile_size.x;
        let left_kill_zone = -KILL_ZONE_BORDER;
        let right_kill_zone = map_width + KILL_ZONE_BORDER;
        let bottom_kill_zone = -KILL_ZONE_BORDER;
        pos.x < left_kill_zone || pos.x > right_kill_zone || pos.y < bottom_kill_zone
    }
}
