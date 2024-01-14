use super::*;

#[derive(HasSchema, Clone, Debug, Default)]
#[type_data(metadata_asset("player"))]
#[repr(C)]
pub struct PlayerMeta {
    pub name: Ustr,
    pub body_size: Vec2,
    pub slide_body_size: Vec2,
    pub gravity: f32,
    pub sounds: PlayerSoundsMeta,
    pub stats: PlayerStatsMeta,
    pub layers: PlayerLayersMeta,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct PlayerLayersMeta {
    pub body: PlayerBodyLayerMeta,
    pub fin: PlayerLayerMeta,
    pub face: PlayerLayerMeta,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct PlayerBodyLayerMeta {
    pub atlas: Handle<Atlas>,
    // #[serde(deserialize_with = "deserialize_body_animations")]
    // #[asset(deserialize_only)]
    pub animations: BodyAnimationsMeta,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[derive_type_data(SchemaDeserialize)]
pub struct BodyAnimationsMeta {
    // TODO: Put Animation Frames in an `Arc` to Avoid Snapshot Clone Cost.
    pub offsets: SMap<Ustr, SVec<Offsets>>,
    pub frames: SMap<Ustr, AnimatedSprite>,
}

impl<'de> Deserialize<'de> for BodyAnimationsMeta {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_body_animations(deserializer)
    }
}

#[derive(Clone, Debug, Default, HasSchema)]
#[repr(C)]
pub struct Offsets {
    pub body: Vec2,
    pub head: Vec2,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct PlayerLayerMeta {
    pub atlas: Handle<Atlas>,
    pub offset: Vec2,
    // TODO: Put Animation Frames in an `Arc` to Avoid Snapshot Clone Cost.
    // The current blocker to this is `Arc` doesn't implement `HasSchema`, and we have to figure
    // that out.
    // TODO: Use more economic key type such as `ustr`
    // The current blocker is implementing `HasSchema` for `ustr`.
    pub animations: SMap<Ustr, AnimatedSprite>,
}

#[derive(HasSchema, Deserialize, Clone, Debug, Default)]
#[repr(C)]
pub struct PlayerStatsMeta {
    pub jump_speed: f32,
    pub slow_fall_speed: f32,
    pub air_speed: f32,
    pub accel_air_speed: f32,
    pub walk_speed: f32,
    pub slowdown: f32,
    pub accel_walk_speed: f32,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct PlayerSoundsMeta {
    pub land_volume: f64,
    pub land: Handle<AudioSource>,

    pub jump_volume: f64,
    pub jump: Handle<AudioSource>,

    pub grab_volume: f64,
    pub grab: Handle<AudioSource>,

    pub drop_volume: f64,
    pub drop: Handle<AudioSource>,
}

fn deserialize_arc<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Arc<T>, D::Error> {
    let item = T::deserialize(deserializer)?;
    Ok(Arc::new(item))
}
fn deserialize_arc_slice<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Arc<[T]>, D::Error> {
    let item = <Vec<T>>::deserialize(deserializer)?;
    Ok(Arc::from(item))
}
fn default_true() -> bool {
    true
}

fn deserialize_body_animations<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BodyAnimationsMeta, D::Error> {
    #[derive(Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct AnimatedBodySprite {
        #[serde(default)]
        pub index: u32,
        #[serde(deserialize_with = "deserialize_arc_slice")]
        #[serde(serialize_with = "serialize_arc_slice")]
        pub frames: Arc<[BodyFrame]>,
        /// The frames per second to play the animation at.
        pub fps: f32,
        /// The amount of time the current frame has been playing
        #[serde(default)]
        pub timer: f32,
        /// Whether or not to repeat the animation
        #[serde(default = "default_true")]
        pub repeat: bool,
    }

    #[derive(Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct BodyFrame {
        pub idx: u32,
        #[serde(default)]
        pub offset: Vec2,
        #[serde(default)]
        pub head_offset: Vec2,
    }

    let body_sprite_anmations = <HashMap<String, AnimatedBodySprite>>::deserialize(deserializer)?;

    let offsets = body_sprite_anmations
        .iter()
        .map(|(k, v)| {
            (
                ustr(k),
                v.frames
                    .iter()
                    .map(|x| Offsets {
                        body: x.offset,
                        head: x.head_offset,
                    })
                    .collect::<SVec<_>>(),
            )
        })
        .collect();
    let frames = body_sprite_anmations
        .into_iter()
        .map(|(k, v)| {
            (
                ustr(&k),
                AnimatedSprite {
                    index: v.index,
                    frames: v.frames.iter().map(|x| x.idx).collect(),
                    fps: v.fps,
                    timer: v.timer,
                    repeat: v.repeat,
                },
            )
        })
        .collect();

    Ok(BodyAnimationsMeta { offsets, frames })
}

/// Metadata for a player emote.
#[derive(HasSchema, Clone, Debug, Default)]
#[type_data(metadata_asset("emote"))]
#[repr(C)]
pub struct EmoteMeta {
    pub name: Ustr,
    pub atlas: Handle<Atlas>,
    pub offset: Vec2,
    pub animation: AnimatedSprite,
}

/// Metadata for a player hat.
#[derive(HasSchema, Clone, Debug, Default)]
#[type_data(metadata_asset("hat"))]
#[repr(C)]
pub struct HatMeta {
    pub name: Ustr,
    pub atlas: Handle<Atlas>,
    pub offset: Vec2,
    pub body_size: Vec2,
}
