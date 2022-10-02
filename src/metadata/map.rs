use bevy_parallax::{LayerData as ParallaxLayerData, ParallaxResource};

use super::*;

#[derive(Component, HasLoadProgress, TypeUuid, Deserialize, Serialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
#[uuid = "8ede98c2-4f17-46f2-bcc5-ae0dc63b2137"]
pub struct MapMeta {
    /// The parallax background layers
    #[serde(default)]
    pub background_layers: Vec<ParallaxLayerMeta>,
    /// Size of the map in tiles
    pub grid_size: UVec2,
    /// The size of the tiles in pixels
    pub tile_size: UVec2,
    /// The layers of the map
    pub layers: Vec<MapLayerMeta>,
}

impl MapMeta {
    #[allow(unused)] // Until we use it
    pub fn get_parallax_resource(&self) -> ParallaxResource {
        ParallaxResource::new(
            self.background_layers
                .iter()
                .cloned()
                .map(Into::into)
                .collect(),
        )
    }
}

#[derive(HasLoadProgress, Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MapLayerMeta {
    pub name: String,
    pub kind: MapLayerKindMeta,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub enum MapLayerKindMeta {
    Tiles {
        tilemap: String,
        #[serde(skip)]
        tilemap_handle: Handle<Image>,
        tiles: Vec<MapTileMeta>,
    },
    Decorations {
        decorations: Vec<MapDecorationMeta>,
    },
}

impl HasLoadProgress for MapLayerKindMeta {}

#[derive(HasLoadProgress, Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MapTileMeta {
    pos: UVec2,
    idx: u32,
}

#[derive(HasLoadProgress, Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MapDecorationMeta {
    pos: UVec2,
    image: String,
    #[serde(skip)]
    image_handle: Handle<Image>,
}

#[derive(HasLoadProgress, Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ParallaxLayerMeta {
    pub speed: f32,
    pub path: String,
    #[serde(skip)]
    pub image_handle: Handle<Image>,
    pub tile_size: Vec2,
    pub cols: usize,
    pub rows: usize,
    pub scale: f32,
    pub z: f32,
    pub transition_factor: f32,
    #[serde(default)]
    pub position: Vec2,
}

impl From<ParallaxLayerMeta> for ParallaxLayerData {
    fn from(meta: ParallaxLayerMeta) -> Self {
        Self {
            speed: meta.speed,
            path: meta.path,
            tile_size: meta.tile_size,
            cols: meta.cols,
            rows: meta.rows,
            scale: meta.scale,
            z: meta.z,
            transition_factor: meta.transition_factor,
            position: meta.position,
        }
    }
}
