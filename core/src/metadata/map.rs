use super::*;
#[derive(BonesBevyAsset, Deserialize, Clone, TypeUlid, Debug)]
#[ulid = "01GP264BT87MAAHMEK52Y5P7BW"]
#[asset_id = "map"]
pub struct MapMeta {
    pub name: String,
    /// The parallax background layers
    #[serde(default)]
    pub background_layers: Vec<ParallaxLayerMeta>,
    /// The background color of the map, behind the parallax layers
    pub background_color: ColorMeta,
    /// Size of the map in tiles
    pub grid_size: UVec2,
    /// The size of the tiles in pixels
    pub tile_size: Vec2,
    /// The layers of the map
    pub layers: Vec<MapLayerMeta>,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct ParallaxLayerMeta {
    pub speed: f32,
    pub image: Handle<Image>,
    pub tile_size: Vec2,
    pub cols: usize,
    pub rows: usize,
    pub scale: f32,
    pub z: f32,
    pub transition_factor: f32,
    pub position: Vec2,
}

impl Default for ParallaxLayerMeta {
    fn default() -> Self {
        Self {
            speed: default(),
            image: default(),
            tile_size: default(),
            cols: 1,
            rows: 1,
            scale: 1.0,
            z: default(),
            transition_factor: 1.0,
            position: default(),
        }
    }
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MapLayerMeta {
    pub id: String,
    pub kind: MapLayerKind,
    #[asset(deserialize_only)]
    #[serde(skip)]
    pub entity: Option<Entity>,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub enum MapLayerKind {
    Tile(MapTileLayer),
    Element(ElementLayer),
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct MapTileLayer {
    pub tilemap: Handle<Atlas>,
    pub has_collision: bool,
    pub tiles: Vec<MapTileMeta>,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct ElementLayer {
    pub elements: Vec<ElementSpawn>,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct ElementSpawn {
    pub pos: Vec2,
    pub element: Handle<ElementMeta>,
}

#[derive(BonesBevyAssetLoad, Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MapTileMeta {
    pub pos: UVec2,
    pub idx: u32,
    #[serde(default)]
    pub jump_through: bool,
}
