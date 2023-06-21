use super::*;

#[derive(BonesBevyAsset, Clone, Debug, Default, Deserialize, TypeUlid)]
#[serde(deny_unknown_fields)]
#[asset_id = "player"]
#[ulid = "01GNWTRN89YRFNFTV3KZZ6H8TK"]
pub struct PlayerMeta {
    pub name: String,
    pub body_size: Vec2,
    pub slide_body_size: Vec2,
    pub gravity: f32,
    pub sounds: PlayerSoundsMeta,
    pub stats: PlayerStatsMeta,
    pub layers: PlayerLayersMeta,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct PlayerLayersMeta {
    pub body: PlayerBodyLayerMeta,
    pub fin: PlayerLayerMeta,
    pub face: PlayerLayerMeta,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct PlayerBodyLayerMeta {
    pub atlas: Handle<Atlas>,
    #[serde(deserialize_with = "deserialize_body_animations")]
    #[asset(deserialize_only)]
    pub animations: BodyAnimationsMeta,
}

#[derive(Clone, Debug, Default)]
pub struct BodyAnimationsMeta {
    pub offsets: Arc<std::collections::HashMap<Key, Vec<Offsets>>>,
    pub frames: Arc<std::collections::HashMap<Key, AnimatedSprite>>,
}

#[derive(Clone, Debug, Default)]
pub struct Offsets {
    pub body: Vec2,
    pub head: Vec2,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct PlayerLayerMeta {
    pub atlas: Handle<Atlas>,
    pub offset: Vec2,
    #[serde(deserialize_with = "deserialize_arc")]
    #[asset(deserialize_only)]
    pub animations: Arc<std::collections::HashMap<Key, AnimatedSprite>>,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct PlayerStatsMeta {
    pub jump_speed: f32,
    pub slow_fall_speed: f32,
    pub air_speed: f32,
    pub accel_air_speed: f32,
    pub walk_speed: f32,
    pub slowdown: f32,
    pub accel_walk_speed: f32,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
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
        pub index: usize,
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
        pub idx: usize,
        #[serde(default)]
        pub offset: Vec2,
        #[serde(default)]
        pub head_offset: Vec2,
    }

    let body_sprite_anmations =
        <std::collections::HashMap<Key, AnimatedBodySprite>>::deserialize(deserializer)?;

    let offsets = Arc::new(
        body_sprite_anmations
            .iter()
            .map(|(k, v)| {
                (
                    *k,
                    v.frames
                        .iter()
                        .map(|x| Offsets {
                            body: x.offset,
                            head: x.head_offset,
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect(),
    );
    let frames = Arc::new(
        body_sprite_anmations
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    AnimatedSprite {
                        index: v.index,
                        frames: v.frames.iter().map(|x| x.idx).collect(),
                        fps: v.fps,
                        timer: v.timer,
                        repeat: v.repeat,
                    },
                )
            })
            .collect(),
    );

    Ok(BodyAnimationsMeta { offsets, frames })
}

/// Metadata for a player hat.
#[derive(BonesBevyAsset, Clone, Debug, Default, Deserialize, TypeUlid)]
#[serde(deny_unknown_fields)]
#[asset_id = "hat"]
#[ulid = "01H3D0EKZAV13T8QXW6SY6S1PP"]
pub struct HatMeta {
    pub name: String,
    pub atlas: Handle<Atlas>,
    pub offset: Vec2,
    pub body_size: Vec2,
}
