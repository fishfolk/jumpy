//! Data structures for things like assets and settings that can be serialized and deserialized.

use crate::prelude::*;

mod localization;
mod settings;
mod ui;

pub use localization::*;
pub use settings::*;
pub use ui::*;

/// Resource containing the main [`Handle<GameMeta>`].
#[derive(Resource, Deref, DerefMut, Clone, Debug, Default)]
pub struct GameMetaHandle(pub Handle<GameMeta>);

#[derive(Resource, BonesBevyAsset, TypeUlid, Deserialize, Clone, Debug)]
#[asset_id = "game"]
#[serde(deny_unknown_fields)]
#[ulid = "01GPEMR6KZ4QZBNN8HBJZEX2JB"]
pub struct GameMeta {
    pub core: AssetHandle<CoreMeta>,
    pub translations: localization::TranslationsMeta,
    pub ui_theme: ui::UIThemeMeta,
    pub main_menu: MainMenuMeta,
    #[asset(deserialize_only)]
    pub default_settings: settings::Settings,
    pub playlist: Vec<AssetHandle<AudioSource>>,
}

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MainMenuMeta {
    pub title_font: FontMeta,
    pub subtitle_font: FontMeta,
    pub background_image: ImageMeta,
    pub menu_width: f32,
}

#[derive(BonesBevyAssetLoad, Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ImageMeta {
    pub image_size: Vec2,
    pub image: AssetHandle<Image>,
    /// Egui texture ID may not be valid on all [`ImageMeta`] unless it is added to the egui context
    /// during game load.
    #[serde(skip)]
    #[asset(deserialize_only)]
    pub egui_texture_id: bevy_egui::egui::TextureId,
}

/// Helper trait for converting color meta to [`egui::Color32`].
pub trait ColorMetaExt {
    fn into_egui(self) -> egui::Color32;
}

impl ColorMetaExt for ColorMeta {
    fn into_egui(self) -> egui::Color32 {
        let [r, g, b, a] = self.0;
        egui::Color32::from_rgba_premultiplied(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        )
    }
}
