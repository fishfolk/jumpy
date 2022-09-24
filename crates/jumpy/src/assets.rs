use std::path::{Path, PathBuf};

use bevy::asset::{Asset, AssetLoader, AssetPath, LoadedAsset};

use crate::{metadata::GameMeta, prelude::*};

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<GameMeta>().add_asset_loader(GameMetaLoader);
    }
}

/// Calculate an asset's full path relative to another asset
fn relative_asset_path(asset_path: &Path, relative_path: &str) -> PathBuf {
    let is_relative = !relative_path.starts_with('/');

    if is_relative {
        let base = asset_path.parent().unwrap_or_else(|| Path::new(""));
        base.join(relative_path)
    } else {
        Path::new(relative_path)
            .strip_prefix("/")
            .unwrap()
            .to_owned()
    }
}

/// Helper to get relative asset paths and handles
fn get_relative_asset<T: Asset>(
    load_context: &bevy::asset::LoadContext,
    self_path: &Path,
    relative_path: &str,
) -> (AssetPath<'static>, Handle<T>) {
    let asset_path = relative_asset_path(self_path, relative_path);
    let asset_path = AssetPath::new(asset_path, None);
    let handle = load_context.get_handle(asset_path.clone());

    (asset_path, handle)
}

#[derive(Default)]
pub struct GameMetaLoader;

impl AssetLoader for GameMetaLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut meta: GameMeta = serde_yaml::from_slice(bytes)?;
            trace!(?meta, "Loaded game asset");

            let self_path = load_context.path().to_owned();

            // Detect the system locale
            let locale = sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string());
            let locale = locale.parse().unwrap_or_else(|e| {
                warn!(
                    "Could not parse system locale string ( \"{}\" ), defaulting to \"en-US\": {}",
                    locale, e
                );
                "en-US".parse().unwrap()
            });
            debug!("Detected system locale: {}", locale);
            meta.translations.detected_locale = locale;

            let mut dependencies = vec![];

            // Get locale handles
            for locale in &meta.translations.locales {
                let (path, handle) = get_relative_asset(load_context, &self_path, locale);
                dependencies.push(path);
                meta.translations.locale_handles.push(handle);
            }

            load_context.set_default_asset(LoadedAsset::new(meta).with_dependencies(dependencies));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["game.yml", "game.yaml"]
    }
}
