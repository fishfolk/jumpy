use crate::animation::Clip;

use super::*;

#[derive(TypeUuid, Deserialize, Clone, Debug, Component)]
#[serde(deny_unknown_fields)]
#[uuid = "a939278b-901a-47d4-8ee8-6ac97881cf4d"]
pub struct PlayerMeta {
    pub name: String,
    pub spritesheet: PlayerSpritesheetMeta,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PlayerSpritesheetMeta {
    pub image: String,
    #[serde(skip)]
    pub atlas_handle: Handle<TextureAtlas>,
    #[serde(skip)]
    pub egui_texture_id: bevy_egui::egui::TextureId,
    pub tile_size: UVec2,
    pub columns: usize,
    pub rows: usize,
    pub animation_fps: f32,
    pub animations: HashMap<String, Clip>,
}
