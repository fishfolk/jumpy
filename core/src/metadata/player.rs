use super::*;

#[derive(BonesBevyAsset, Clone, Debug, Default, Deserialize, TypeUlid)]
#[asset_id = "player"]
#[ulid = "01GNWTRN89YRFNFTV3KZZ6H8TK"]
pub struct PlayerMeta {
    pub name: String,
    pub atlas: Handle<Atlas>,
    pub body_size: Vec2,
    pub gravity: f32,
    pub sounds: PlayerSounds,
    pub stats: PlayerStats,

    #[serde(deserialize_with = "deserialize_arc")]
    #[asset(deserialize_only)]
    pub animations: Arc<std::collections::HashMap<Key, AnimatedSprite>>,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct PlayerStats {
    pub jump_speed: f32,
    pub slow_fall_speed: f32,
    pub air_move_speed: f32,
    pub walk_speed: f32,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct PlayerSounds {
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
