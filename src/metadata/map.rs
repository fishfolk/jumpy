use crate::animation::AnimatedSprite;

use super::*;

use bevy::reflect::FromReflect;
use bevy_parallax::{LayerData as ParallaxLayerData, ParallaxResource};

pub struct MapMetadataPlugin;

impl Plugin for MapMetadataPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MapMeta>()
            .register_type::<Vec<String>>()
            .register_type::<HashMap<String, AnimatedSprite>>()
            .register_type::<MapElementMeta>();
    }
}

#[derive(
    Reflect, Component, HasLoadProgress, TypeUuid, Deserialize, Serialize, Clone, Debug, Default,
)]
#[reflect(Component, Default)]
#[serde(deny_unknown_fields)]
#[uuid = "8ede98c2-4f17-46f2-bcc5-ae0dc63b2137"]
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

#[derive(Reflect, FromReflect, HasLoadProgress, Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MapLayerMeta {
    pub id: String,
    pub kind: MapLayerKind,
    #[serde(skip)]
    pub entity: Option<Entity>,
}

#[derive(Reflect, FromReflect, Deserialize, Serialize, Clone, Debug)]
#[reflect_value(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub enum MapLayerKind {
    Tile(MapTileLayer),
    Element(MapElementLayer),
}

impl HasLoadProgress for MapLayerKind {
    fn load_progress(
        &self,
        loading_resources: &bones_has_load_progress::LoadingResources,
    ) -> bones_has_load_progress::LoadProgress {
        match self {
            MapLayerKind::Tile(tile_layer) => tile_layer.load_progress(loading_resources),
            MapLayerKind::Element(element_layer) => element_layer.load_progress(loading_resources),
        }
    }
}

#[derive(Reflect, HasLoadProgress, Deserialize, Serialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct MapTileLayer {
    pub tilemap: String,
    #[serde(skip)]
    pub tilemap_handle: AssetHandle<Image>,
    pub has_collision: bool,
    pub tiles: Vec<MapTileMeta>,
}

#[derive(HasLoadProgress, Deserialize, Serialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct MapElementLayer {
    pub elements: Vec<MapElementSpawn>,
}

#[derive(HasLoadProgress, Deserialize, Serialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct MapElementSpawn {
    pub pos: Vec2,
    pub element: String,
    #[serde(skip)]
    pub element_handle: AssetHandle<MapElementMeta>,
}

#[derive(Reflect, FromReflect, HasLoadProgress, Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MapTileMeta {
    pub pos: UVec2,
    pub idx: u32,
    #[serde(default)]
    pub jump_through: bool,
}

#[derive(Reflect, FromReflect, HasLoadProgress, Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct ParallaxLayerMeta {
    pub speed: f32,
    pub image: String,
    #[serde(skip)]
    pub image_handle: AssetHandle<Image>,
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
            image_handle: default(),
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

impl From<ParallaxLayerMeta> for ParallaxLayerData {
    fn from(meta: ParallaxLayerMeta) -> Self {
        Self {
            speed: meta.speed,
            path: meta.image,
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

#[derive(
    Reflect, Component, HasLoadProgress, TypeUuid, Deserialize, Serialize, Clone, Debug, Default,
)]
#[reflect(Default, Component)]
#[serde(deny_unknown_fields)]
#[uuid = "0a4a0cc6-ee52-4b0d-a88b-871c49a06622"]
pub struct MapElementMeta {
    pub name: String,
    pub category: String,
    // #[serde(default)]
    // pub scripts: Vec<String>,
    #[serde(default)]
    #[has_load_progress(none)]
    pub builtin: BuiltinElementKind,

    /// The size of the bounding rect for the element in the editor
    #[serde(default = "editor_size_default")]
    pub editor_size: Vec2,

    // #[serde(skip)]
    // pub script_handles: Vec<AssetHandle<JsScript>>,
    /// Assets that should be pre-loaded by the game before starting
    #[serde(default)]
    pub preload_assets: Vec<String>,
    #[serde(skip)]
    #[reflect(ignore)]
    pub preload_asset_handles: Vec<HandleUntyped>,
}

fn editor_size_default() -> Vec2 {
    Vec2::splat(16.0)
}

/// The kind of built-in
#[derive(Reflect, Component, Deserialize, Serialize, Clone, Debug, Default)]
#[reflect(Default, Component)]
#[serde(deny_unknown_fields)]
pub enum BuiltinElementKind {
    /// This is not a built-in item
    #[default]
    None,
    /// Player spawner
    PlayerSpawner,
    /// Grenades item
    Grenade {
        body_size: Vec2,
        grab_offset: Vec2,
        damage_region_size: Vec2,
        damage_region_lifetime: f32,
        throw_velocity: Vec2,
        explosion_lifetime: f32,
        explosion_frames: usize,
        explosion_fps: f32,
        explosion_sound: String,
        #[serde(skip)]
        explosion_sound_handle: Handle<AudioSource>,
        fuse_sound: String,
        #[serde(skip)]
        fuse_sound_handle: Handle<AudioSource>,
        /// The time in seconds before a grenade explodes
        fuse_time: f32,
        #[serde(default)]
        can_rotate: bool,
        /// The grenade atlas
        atlas: String,
        #[serde(skip)]
        atlas_handle: AssetHandle<TextureAtlas>,
        explosion_atlas: String,
        #[serde(skip)]
        explosion_atlas_handle: AssetHandle<TextureAtlas>,
        #[serde(default)]
        bounciness: f32,
        #[serde(default)]
        angular_velocity: f32,
    },
    /// An animated decoration such as seaweed or anemones
    AnimatedDecoration {
        start_frame: usize,
        end_frame: usize,
        fps: f32,
        atlas: String,
        #[serde(skip)]
        atlas_handle: AssetHandle<TextureAtlas>,
    },
    /// A crab roaming on the ocean floor
    Crab {
        start_frame: usize,
        end_frame: usize,
        fps: f32,
        atlas: String,
        comfortable_spawn_distance: f32,
        comfortable_scared_distance: f32,
        same_level_threshold: f32,
        walk_speed: f32,
        run_speed: f32,
        /// 45 fix updates per second, so if this is 45 the maximum delay between actions
        /// will be 1 second
        timer_delay_max: u8,
        #[serde(skip)]
        atlas_handle: AssetHandle<TextureAtlas>,
    },
    /// This is a sproinger
    Sproinger {
        atlas: String,
        #[serde(skip)]
        atlas_handle: AssetHandle<TextureAtlas>,
        sound: String,
        #[serde(skip)]
        sound_handle: Handle<AudioSource>,
    },
    /// This is a sword
    Sword {
        atlas: String,
        #[serde(skip)]
        atlas_handle: AssetHandle<TextureAtlas>,
        sound: String,
        #[serde(skip)]
        sound_handle: Handle<AudioSource>,
        #[serde(default)]
        angular_velocity: f32,
        #[serde(default)]
        can_rotate: bool,
        #[serde(default)]
        arm_delay: f32,
        throw_velocity: Vec2,
        #[serde(default)]
        bounciness: f32,
    },
    /// The throwable crate item
    Crate {
        atlas: String,
        #[serde(skip)]
        atlas_handle: AssetHandle<TextureAtlas>,

        breaking_atlas: String,
        #[serde(skip)]
        breaking_atlas_handle: AssetHandle<TextureAtlas>,
        breaking_anim_frames: usize,
        breaking_anim_fps: f32,

        break_sound: String,
        #[serde(skip)]
        break_sound_handle: Handle<AudioSource>,

        throw_velocity: Vec2,

        body_size: Vec2,
        grab_offset: Vec2,
        // How long to wait before despawning a thrown crate, if it hans't it anything yet.
        break_timeout: f32,
    },
    /// The mine item
    Mine {
        atlas: String,
        #[serde(skip)]
        atlas_handle: AssetHandle<TextureAtlas>,

        damage_region_size: Vec2,
        damage_region_lifetime: f32,
        explosion_atlas: String,
        #[serde(skip)]
        explosion_atlas_handle: AssetHandle<TextureAtlas>,
        explosion_anim_frames: usize,
        explosion_anim_fps: f32,

        arm_sound: String,
        armed_anim_start: usize,
        armed_anim_end: usize,
        armed_anim_fps: f32,
        #[serde(skip)]
        arm_sound_handle: Handle<AudioSource>,
        explosion_sound: String,
        #[serde(skip)]
        explosion_sound_handle: Handle<AudioSource>,

        throw_velocity: Vec2,
        /// The delay after throwing the mine, before it becomes armed and will blow up on contact.
        arm_delay: f32,

        body_size: Vec2,
        grab_offset: Vec2,
    },

    StompBoots {
        map_icon: String,
        #[serde(skip)]
        map_icon_handle: Handle<TextureAtlas>,

        player_decoration: String,
        #[serde(skip)]
        player_decoration_handle: Handle<TextureAtlas>,

        body_size: Vec2,
        grab_offset: Vec2,
    },
    KickBomb {
        body_size: Vec2,
        grab_offset: Vec2,
        damage_region_size: Vec2,
        damage_region_lifetime: f32,
        throw_velocity: Vec2,
        explosion_lifetime: f32,
        explosion_frames: usize,
        explosion_fps: f32,
        explosion_sound: String,
        #[serde(skip)]
        explosion_sound_handle: Handle<AudioSource>,
        fuse_sound: String,
        #[serde(skip)]
        fuse_sound_handle: Handle<AudioSource>,
        /// The time in seconds before a grenade explodes
        fuse_time: f32,
        #[serde(default)]
        can_rotate: bool,
        /// The grenade atlas
        atlas: String,
        #[serde(skip)]
        atlas_handle: AssetHandle<TextureAtlas>,
        explosion_atlas: String,
        #[serde(skip)]
        explosion_atlas_handle: AssetHandle<TextureAtlas>,
        #[serde(default)]
        bounciness: f32,
        #[serde(default)]
        angular_velocity: f32,
        #[serde(default)]
        arm_delay: f32,
    },
}
