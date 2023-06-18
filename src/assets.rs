//! Custom asset loaders and handle types.

use bevy::{
    asset::{AssetLoader, LoadedAsset},
    reflect::TypeUuid,
};
use bevy_egui::egui;
use bones_bevy_asset::BonesBevyAssetAppExt;

use crate::{metadata::GameMeta, prelude::*};

#[doc(hidden)]
mod asset_handle;
pub use asset_handle::AssetHandle;

/// Asset plugin.
pub struct JumpyAssetPlugin;

impl Plugin for JumpyAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_bones_asset::<GameMeta>()
            .add_asset::<EguiFont>()
            .add_asset_loader(EguiFontLoader);
    }
}

/// Asset type containing [`egui::FontData`][crate::external::egui::FontData].
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "421c9e38-89be-43ff-a293-6fea65abf946"]
pub struct EguiFont(pub egui::FontData);

/// Bevy asset loader for [`EguiFont`] assets.
pub struct EguiFontLoader;

impl AssetLoader for EguiFontLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let path = load_context.path();
            let data = egui::FontData::from_owned(bytes.to_vec());
            trace!(?path, "Loaded font asset");

            load_context.set_default_asset(LoadedAsset::new(EguiFont(data)));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ttf"]
    }
}
