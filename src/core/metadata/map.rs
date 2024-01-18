use super::*;

#[derive(Clone, HasSchema, Default, Debug)]
#[type_data(metadata_asset("map"))]
#[repr(C)]
pub struct MapMeta {
    pub name: Ustr,
    /// The parallax background layers
    pub background: BackgroundMeta,
    /// The background color of the map, behind the parallax layers
    pub background_color: Color,
    /// Size of the map in tiles
    pub grid_size: UVec2,
    /// The size of the tiles in pixels
    pub tile_size: Vec2,
    /// The layers of the map
    pub layers: SVec<MapLayerMeta>,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct BackgroundMeta {
    pub speed: Vec2,
    pub layers: SVec<ParallaxLayerMeta>,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct ParallaxLayerMeta {
    pub image: Handle<Image>,
    pub size: Vec2,
    pub depth: f32,
    pub scale: f32,
    pub offset: Vec2,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct MapLayerMeta {
    pub id: Ustr,
    pub tilemap: Maybe<Handle<Atlas>>,
    pub tiles: SVec<MapTileMeta>,
    pub elements: SVec<ElementSpawn>,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct ElementSpawn {
    pub pos: Vec2,
    pub element: Handle<ElementMeta>,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct MapTileMeta {
    pub pos: UVec2,
    pub idx: u32,
    pub collision: TileCollisionKind,
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
