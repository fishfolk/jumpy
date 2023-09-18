use std::time::Duration;

use super::*;

#[derive(HasSchema, Default, Clone, Debug)]
#[type_data(metadata_asset("element"))]
#[repr(C)]
pub struct ElementMeta {
    pub name: Ustr,
    pub category: Ustr,
    pub data: Handle<SchemaBox>,
    pub editor: ElementEditorMeta,
}

#[derive(HasSchema, Deserialize, Clone, Debug)]
#[repr(C)]
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

#[derive(HasSchema, Clone, Debug, Default)]
#[type_data(metadata_asset("bullet"))]
pub struct BulletMeta {
    pub velocity: Vec2,
    pub body_diameter: f32,
    pub atlas: Handle<Atlas>,

    pub lifetime: f32,
    pub explosion_fps: f32,
    pub explosion_volume: f64,
    pub explosion_lifetime: f32,
    pub explosion_frames: u32,
    pub explosion_atlas: Handle<Atlas>,
    pub explosion_sound: Handle<AudioSource>,
}

/// Player spawner element
#[derive(HasSchema, Default, Debug, Clone, Copy)]
#[type_data(metadata_asset("player_spawner"))]
#[repr(C)]
pub struct PlayerSpawnerMeta;

/// Grenades item
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("grenade"))]
#[repr(C)]
pub struct GrenadeMeta {
    pub body_diameter: f32,
    pub fin_anim: Ustr,
    pub grab_offset: Vec2,
    pub damage_region_size: Vec2,
    pub damage_region_lifetime: f32,
    pub throw_velocity: f32,
    pub explosion_lifetime: f32,
    pub explosion_frames: u32,
    pub explosion_fps: f32,
    pub explosion_sound: Handle<AudioSource>,
    pub explosion_volume: f64,
    pub fuse_sound: Handle<AudioSource>,
    pub fuse_sound_volume: f64,
    /// The time in seconds before a grenade explodes
    pub fuse_time: f32,
    pub can_rotate: bool,
    /// The grenade atlas
    pub atlas: Handle<Atlas>,
    pub explosion_atlas: Handle<Atlas>,
    pub bounciness: f32,
    pub angular_velocity: f32,
}

/// An animated decoration such as seaweed or anemones
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("animated_decoration"))]
#[repr(C)]
pub struct AnimatedDecorationMeta {
    pub start_frame: u32,
    pub end_frame: u32,
    pub fps: f32,
    pub atlas: Handle<Atlas>,
}

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("fish_school"))]
#[repr(C)]
pub struct FishSchoolMeta {
    pub kinds: SVec<Handle<Atlas>>,
    /// The default and most-likely to ocurr number of fish in a school
    pub base_count: u32,
    /// The ammount greater or less than the base number of fish that may spawn
    pub count_variation: u32,
    /// The distance from the spawn point on each axis that the individual fish in the school will be
    /// initially spawned within
    pub spawn_range: f32,
    /// The distance that the fish wish to stay within the center of their school
    pub school_size: f32,
    // The distance a collider must be for the fish to run away
    pub flee_range: f32,
}
/// A crab roaming on the ocean floor
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("crab"))]
#[repr(C)]
pub struct CrabMeta {
    pub body_size: Vec2,
    pub walk_frames: SVec<u32>,
    pub spawn_frames: SVec<u32>,
    pub fps: f32,
    pub comfortable_spawn_distance: f32,
    pub comfortable_scared_distance: f32,
    /// How long a crab has to be away from it's spawn point before it digs into the ground and
    /// digs back out in his spawn point.
    pub uncomfortable_respawn_time: Duration,
    pub same_level_threshold: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    // TODO: migrate this to a duration like `uncomfortable_respawn_time`.
    pub timer_delay_max: u8,
    pub atlas: Handle<Atlas>,
}
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("snail"))]
#[repr(C)]
pub struct SnailMeta {
    pub atlas: Handle<Atlas>,
    pub fps: f32,
    pub body_diameter: f32,
    pub bounciness: f32,
    pub gravity: f32,
    pub hit_speed: f32,
    /// The animation frames for when the snail is crawling
    pub crawl_frames: SVec<u32>,
    /// The `crawl_frames` indexes in which to move the snail
    pub move_frame_indexes: SVec<u32>,
    /// The animation frames for when the snail is fleeing into its shell.
    ///
    /// **Note:** This is reversed for the snail coming out of its shell.
    pub hide_frames: SVec<u32>,
    pub hide_time: f32,
}
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("urchin"))]
#[repr(C)]
pub struct UrchinMeta {
    pub image: Handle<Image>,
    pub body_diameter: f32,
    pub hit_speed: f32,
    pub gravity: f32,
    pub bounciness: f32,
    pub spin: f32,
}
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("sproinger"))]
#[repr(C)]
/// This is a sproinger
pub struct SproingerMeta {
    pub atlas: Handle<Atlas>,
    pub sound: Handle<AudioSource>,
    pub sound_volume: f64,
    pub body_size: Vec2,
    pub spring_velocity: f32,
}
/// This is a sword
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("sword"))]
#[repr(C)]
pub struct SwordMeta {
    pub atlas: Handle<Atlas>,
    pub sound: Handle<AudioSource>,
    pub sound_volume: f64,
    pub body_size: Vec2,
    pub fin_anim: Ustr,
    pub grab_offset: Vec2,
    pub killing_speed: f32,
    pub angular_velocity: f32,
    pub can_rotate: bool,
    pub bounciness: f32,
    pub throw_velocity: f32,
    pub cooldown_frames: u32,
}
/// The throwable crate item
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("crate"))]
#[repr(C)]
pub struct CrateMeta {
    pub atlas: Handle<Atlas>,

    pub breaking_atlas: Handle<Atlas>,
    pub breaking_anim_frames: u32,
    pub breaking_anim_fps: f32,

    pub break_sound: Handle<AudioSource>,
    pub break_sound_volume: f64,
    pub bounce_sound: Handle<AudioSource>,
    pub bounce_sound_volume: f64,

    pub throw_velocity: f32,

    pub body_size: Vec2,
    pub grab_offset: Vec2,
    // How long to wait before despawning a thrown crate, if it hans't it anything yet.
    pub break_timeout: Duration,
    pub bounciness: f32,
    pub fin_anim: Ustr,
    pub crate_break_state_1: u32,
    pub crate_break_state_2: u32,
}
/// The mine item
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("mine"))]
#[repr(C)]
pub struct MineMeta {
    pub atlas: Handle<Atlas>,

    pub damage_region_size: Vec2,
    pub damage_region_lifetime: f32,
    pub explosion_atlas: Handle<Atlas>,
    pub explosion_lifetime: f32,
    pub explosion_frames: u32,
    pub explosion_fps: f32,
    pub explosion_volume: f64,
    pub explosion_sound: Handle<AudioSource>,

    /// The delay after throwing the mine, before it becomes armed and will blow up on contact.
    pub arm_delay: f32,
    pub armed_frames: u32,
    pub armed_fps: f32,
    pub arm_sound_volume: f64,
    pub arm_sound: Handle<AudioSource>,

    pub throw_velocity: f32,
    pub body_size: Vec2,
    pub grab_offset: Vec2,
    pub fin_anim: Ustr,
    pub bounciness: f32,
}

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("stomp_boots"))]
#[repr(C)]
pub struct StompBootsMeta {
    pub map_icon: Handle<Atlas>,
    pub player_decoration: Handle<Atlas>,

    pub body_size: Vec2,
    pub grab_offset: Vec2,
}
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("kick_bomb"))]
#[repr(C)]
pub struct KickBombMeta {
    pub body_diameter: f32,
    pub fin_anim: Ustr,
    pub grab_offset: Vec2,
    pub damage_region_size: Vec2,
    pub damage_region_lifetime: f32,
    pub kick_velocity: Vec2,
    pub throw_velocity: f32,
    pub explosion_lifetime: f32,
    pub explosion_frames: u32,
    pub explosion_fps: f32,
    pub explosion_sound: Handle<AudioSource>,
    pub explosion_volume: f64,
    pub fuse_sound: Handle<AudioSource>,
    pub fuse_sound_volume: f64,
    /// The time in seconds before a grenade explodes
    pub fuse_time: Duration,
    pub can_rotate: bool,
    /// The grenade atlas
    pub atlas: Handle<Atlas>,
    pub explosion_atlas: Handle<Atlas>,
    pub bounciness: f32,
    pub angular_velocity: f32,
    pub arm_delay: Duration,
}

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("musket"))]
#[repr(C)]
pub struct MusketMeta {
    pub grab_offset: Vec2,
    pub fin_anim: Ustr,

    pub body_size: Vec2,
    pub bounciness: f32,
    pub can_rotate: bool,
    pub throw_velocity: f32,
    pub angular_velocity: f32,
    pub atlas: Handle<Atlas>,

    pub max_ammo: u32,
    pub cooldown: Duration,
    pub bullet_meta: Handle<BulletMeta>,
    pub kickback: f32,

    pub shoot_fps: f32,
    pub shoot_lifetime: f32,
    pub shoot_frames: u32,
    pub shoot_sound_volume: f64,
    pub empty_shoot_sound_volume: f64,
    pub shoot_atlas: Handle<Atlas>,
    pub shoot_sound: Handle<AudioSource>,
    pub empty_shoot_sound: Handle<AudioSource>,
}
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("slippery_seaweed"))]
#[repr(C)]
pub struct SlipperySeaweedMeta {
    pub atlas: Handle<Atlas>,
    pub start_frame: u32,
    pub end_frame: u32,
    pub fps: f32,
    pub body_size: Vec2,
}

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("slippery"))]
#[repr(C)]
pub struct SlipperyMeta {
    pub atlas: Handle<Atlas>,
    pub body_size: Vec2,
    pub player_slide: f32,
    pub body_friction: f32,
}

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("spike"))]
#[repr(C)]
pub struct SpikeMeta {
    pub atlas: Handle<Atlas>,
    pub body_size: Vec2,
    pub start_frame: u32,
    pub end_frame: u32,
    pub fps: f32,
}
