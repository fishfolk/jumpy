use super::*;

#[derive(BonesBevyAsset, TypeUlid, Deserialize, Clone, Debug, Default)]
#[asset_id = "element"]
#[ulid = "01GP28EQQVVQHDA0C9C4168C7W"]
#[serde(deny_unknown_fields)]
pub struct ElementMeta {
    pub name: String,
    pub category: String,
    #[serde(default)]
    pub builtin: BuiltinElementKind,

    /// The size of the bounding rect for the element in the editor
    #[serde(default = "editor_size_default")]
    pub editor_size: Vec2,
}

fn editor_size_default() -> Vec2 {
    Vec2::splat(16.0)
}

/// The kind of built-in
#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug, Default)]
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
        body_offset: Vec2,
        grab_offset: Vec2,
        damage_region_size: Vec2,
        damage_region_lifetime: f32,
        throw_velocity: Vec2,
        explosion_lifetime: f32,
        explosion_frames: usize,
        explosion_fps: f32,
        explosion_sound: Handle<AudioSource>,
        fuse_sound: String,
        #[serde(skip)]
        fuse_sound_handle: Handle<AudioSource>,
        /// The time in seconds before a grenade explodes
        fuse_time: f32,
        #[serde(default)]
        can_rotate: bool,
        /// The grenade atlas
        atlas: Handle<Atlas>,
        explosion_atlas: Handle<Atlas>,
        #[serde(default)]
        bouncyness: f32,
        #[serde(default)]
        angular_velocity: f32,
    },
    /// An animated decoration such as seaweed or anemones
    AnimatedDecoration {
        start_frame: usize,
        end_frame: usize,
        fps: f32,
        atlas: Handle<Atlas>,
    },
    /// A crab roaming on the ocean floor
    Crab {
        start_frame: usize,
        end_frame: usize,
        fps: f32,
        comfortable_spawn_distance: f32,
        comfortable_scared_distance: f32,
        same_level_threshold: f32,
        walk_speed: f32,
        run_speed: f32,
        /// 45 fix updates per second, so if this is 45 the maximum delay between actions
        /// will be 1 second
        timer_delay_max: u8,
        atlas: Handle<Atlas>,
    },
    /// This is a sproinger
    Sproinger {
        atlas: Handle<Atlas>,
        sound: Handle<AudioSource>,
        sound_volume: f32,
        body_size: Vec2,
        body_offset: Vec2,
        spring_velocity: f32,
    },
    /// This is a sword
    Sword {
        atlas: Handle<Atlas>,
        sound: Handle<AudioSource>,
        sound_volume: f32,
        body_size: Vec2,
        #[serde(default)]
        body_offset: Vec2,
        angular_velocity: f32,
        can_rotate: bool,
        arm_delay: f32,
        bounciness: f32,
        throw_velocity: Vec2,
        cooldown_frames: usize,
    },
    /// The throwable crate item
    Crate {
        atlas: Handle<Atlas>,

        breaking_atlas: Handle<Atlas>,
        breaking_anim_frames: usize,
        breaking_anim_fps: f32,

        break_sound: Handle<AudioSource>,

        throw_velocity: Vec2,

        body_size: Vec2,
        body_offset: Vec2,
        grab_offset: Vec2,
        // How long to wait before despawning a thrown crate, if it hans't it anything yet.
        break_timeout: f32,
    },
    /// The mine item
    Mine {
        atlas: Handle<Atlas>,

        damage_region_size: Vec2,
        damage_region_lifetime: f32,
        explosion_atlas: Handle<Atlas>,
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
        body_offset: Vec2,
        grab_offset: Vec2,
    },

    StompBoots {
        map_icon: Handle<Atlas>,
        player_decoration: Handle<Atlas>,

        body_size: Vec2,
        body_offset: Vec2,
        grab_offset: Vec2,
    },
    KickBomb {
        body_size: Vec2,
        body_offset: Vec2,
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
        atlas: Handle<Atlas>,
        explosion_atlas: Handle<Atlas>,
        #[serde(default)]
        bouncyness: f32,
        #[serde(default)]
        angular_velocity: f32,
        #[serde(default)]
        arm_delay: f32,
    },
}
