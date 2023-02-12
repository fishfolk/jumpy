use super::*;

#[derive(BonesBevyAsset, Clone, Debug, Default, Deserialize, TypeUlid)]
#[serde(deny_unknown_fields)]
#[asset_id = "player"]
#[ulid = "01GNWTRN89YRFNFTV3KZZ6H8TK"]
pub struct PlayerMeta {
    pub name: String,
    pub body_size: Vec2,
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
    pub body_offsets: Arc<std::collections::HashMap<Key, Vec<Vec2>>>,
    pub frames: Arc<std::collections::HashMap<Key, AnimatedSprite>>,
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
    pub land_volume: f32,
    pub land: Handle<AudioSource>,

    pub jump_volume: f32,
    pub jump: Handle<AudioSource>,

    pub grab_volume: f32,
    pub grab: Handle<AudioSource>,

    pub drop_volume: f32,
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
    }

    let body_sprite_anmations =
        <std::collections::HashMap<Key, AnimatedBodySprite>>::deserialize(deserializer)?;

    let body_offsets = Arc::new(
        body_sprite_anmations
            .iter()
            .map(|(k, v)| (*k, v.frames.iter().map(|x| x.offset).collect::<Vec<_>>()))
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

    Ok(BodyAnimationsMeta {
        body_offsets,
        frames,
    })
}
