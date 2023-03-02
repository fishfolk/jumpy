use std::time::Duration;

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

    #[serde(default)]
    pub editor: ElementEditorMeta,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct ElementEditorMeta {
    /// The size of the bounding rect for the element in the editor
    pub grab_size: Vec2,
    /// The offset of the bounding rect for the element in the editor.
    pub grab_offset: Vec2,
    /// Show the element name above the bounding rect in the editor.
    pub show_name: bool,
}

impl Default for ElementEditorMeta {
    fn default() -> Self {
        Self {
            grab_size: Vec2::splat(45.0),
            grab_offset: Vec2::ZERO,
            show_name: true,
        }
    }
}

#[derive(BonesBevyAsset, Deserialize, Clone, Debug, Default, TypeUlid)]
#[ulid = "01GR1W2B3S7DM5QEY07RSJH2G0"]
#[asset_id = "bullet"]
#[serde(deny_unknown_fields)]
pub struct BulletMeta {
    pub velocity: Vec2,
    pub body_diameter: f32,
    pub atlas: Handle<Atlas>,

    pub lifetime: f32,
    pub explosion_fps: f32,
    pub explosion_volume: f32,
    pub explosion_lifetime: f32,
    pub explosion_frames: usize,
    pub explosion_atlas: Handle<Atlas>,
    pub explosion_sound: Handle<AudioSource>,
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
        body_diameter: f32,
        fin_anim: Key,
        grab_offset: Vec2,
        damage_region_size: Vec2,
        damage_region_lifetime: f32,
        throw_velocity: f32,
        explosion_lifetime: f32,
        explosion_frames: usize,
        explosion_fps: f32,
        explosion_sound: Handle<AudioSource>,
        explosion_volume: f32,
        fuse_sound: Handle<AudioSource>,
        fuse_sound_volume: f32,
        /// The time in seconds before a grenade explodes
        fuse_time: f32,
        #[serde(default)]
        can_rotate: bool,
        /// The grenade atlas
        atlas: Handle<Atlas>,
        explosion_atlas: Handle<Atlas>,
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
        atlas: Handle<Atlas>,
    },
    FishSchool {
        kinds: Vec<Handle<Atlas>>,
        /// The default and most-likely to ocurr number of fish in a school
        base_count: u32,
        /// The ammount greater or less than the base number of fish that may spawn
        count_variation: u32,
        /// The distance from the spawn point on each axis that the individual fish in the school will be
        /// initially spawned within
        spawn_range: f32,
        /// The distance that the fish wish to stay within the center of their school
        school_size: f32,
        // The distance a collider must be for the fish to run away
        flee_range: f32,
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
    Urchin {
        image: Handle<Image>,
        body_diameter: f32,
        hit_speed: f32,
        gravity: f32,
        bounciness: f32,
        spin: f32,
    },
    /// This is a sproinger
    Sproinger {
        atlas: Handle<Atlas>,
        sound: Handle<AudioSource>,
        sound_volume: f32,
        body_size: Vec2,
        spring_velocity: f32,
    },
    /// This is a sword
    Sword {
        atlas: Handle<Atlas>,
        sound: Handle<AudioSource>,
        sound_volume: f32,
        body_size: Vec2,
        fin_anim: Key,
        #[serde(default)]
        grab_offset: Vec2,
        killing_speed: f32,
        angular_velocity: f32,
        can_rotate: bool,
        bounciness: f32,
        throw_velocity: f32,
        cooldown_frames: usize,
    },
    /// The throwable crate item
    Crate {
        atlas: Handle<Atlas>,

        breaking_atlas: Handle<Atlas>,
        breaking_anim_frames: usize,
        breaking_anim_fps: f32,

        break_sound: Handle<AudioSource>,
        break_sound_volume: f32,
        bounce_sound: Handle<AudioSource>,
        bounce_sound_volume: f32,

        throw_velocity: f32,

        body_size: Vec2,
        grab_offset: Vec2,
        // How long to wait before despawning a thrown crate, if it hans't it anything yet.
        break_timeout: f32,
        bounciness: f32,
        fin_anim: Key,
        crate_break_state_1: usize,
        crate_break_state_2: usize,
    },
    /// The mine item
    Mine {
        atlas: Handle<Atlas>,

        damage_region_size: Vec2,
        damage_region_lifetime: f32,
        explosion_atlas: Handle<Atlas>,
        explosion_lifetime: f32,
        explosion_frames: usize,
        explosion_fps: f32,
        explosion_volume: f32,
        explosion_sound: Handle<AudioSource>,

        /// The delay after throwing the mine, before it becomes armed and will blow up on contact.
        arm_delay: f32,
        armed_frames: usize,
        armed_fps: f32,
        arm_sound_volume: f32,
        arm_sound: Handle<AudioSource>,

        throw_velocity: f32,
        body_size: Vec2,
        grab_offset: Vec2,
        fin_anim: Key,
        bounciness: f32,
    },

    StompBoots {
        map_icon: Handle<Atlas>,
        player_decoration: Handle<Atlas>,

        body_size: Vec2,
        grab_offset: Vec2,
    },
    KickBomb {
        body_diameter: f32,
        fin_anim: Key,
        grab_offset: Vec2,
        damage_region_size: Vec2,
        damage_region_lifetime: f32,
        kick_velocity: Vec2,
        throw_velocity: f32,
        explosion_lifetime: f32,
        explosion_frames: usize,
        explosion_fps: f32,
        explosion_sound: Handle<AudioSource>,
        explosion_volume: f32,
        fuse_sound: Handle<AudioSource>,
        fuse_sound_volume: f32,
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
        bounciness: f32,
        #[serde(default)]
        angular_velocity: f32,
        #[serde(default)]
        arm_delay: f32,
    },
    Musket {
        #[serde(default)]
        grab_offset: Vec2,
        fin_anim: Key,

        body_size: Vec2,
        bounciness: f32,
        can_rotate: bool,
        throw_velocity: f32,
        angular_velocity: f32,
        atlas: Handle<Atlas>,

        max_ammo: usize,
        #[serde(with = "humantime_serde")]
        cooldown: Duration,
        bullet_meta: Handle<BulletMeta>,

        shoot_fps: f32,
        shoot_lifetime: f32,
        shoot_frames: usize,
        shoot_sound_volume: f32,
        empty_shoot_sound_volume: f32,
        shoot_atlas: Handle<Atlas>,
        shoot_sound: Handle<AudioSource>,
        empty_shoot_sound: Handle<AudioSource>,
    },
    SlipperySeaweed {
        atlas: Handle<Atlas>,
        start_frame: usize,
        end_frame: usize,
        fps: f32,
        body_size: Vec2,
    },
}
