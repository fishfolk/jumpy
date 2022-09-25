use std::path::{Path, PathBuf};

use bevy::{
    asset::{Asset, AssetLoader, AssetPath, LoadedAsset},
    reflect::TypeUuid,
};
use bevy_egui::egui;

use crate::{
    metadata::{ui::BorderImageMeta, GameMeta},
    prelude::*,
};

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<GameMeta>()
            .add_asset_loader(GameMetaLoader)
            .add_asset::<EguiFont>()
            .add_asset_loader(EguiFontLoader);
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

            // Load the main menu background
            let (main_menu_background_path, main_menu_background) = get_relative_asset(
                load_context,
                &self_path,
                &meta.main_menu.background_image.image,
            );
            meta.main_menu.background_image.image_handle = main_menu_background;
            dependencies.push(main_menu_background_path);

            // Load UI border images
            let mut load_border_image = |border: &mut BorderImageMeta| {
                let (path, handle) = get_relative_asset(load_context, &self_path, &border.image);
                dependencies.push(path);
                border.handle = handle;
            };
            load_border_image(&mut meta.ui_theme.hud.portrait_frame);
            load_border_image(&mut meta.ui_theme.panel.border);
            load_border_image(&mut meta.ui_theme.hud.lifebar.background_image);
            load_border_image(&mut meta.ui_theme.hud.lifebar.progress_image);
            for button in meta.ui_theme.button_styles.values_mut() {
                load_border_image(&mut button.borders.default);
                if let Some(border) = &mut button.borders.clicked {
                    load_border_image(border);
                }
                if let Some(border) = &mut button.borders.focused {
                    load_border_image(border);
                }
            }

            // Load UI fonts
            for (font_name, font_relative_path) in &meta.ui_theme.font_families {
                let (font_path, font_handle) =
                    get_relative_asset(load_context, &self_path, font_relative_path);

                dependencies.push(font_path);

                meta.ui_theme
                    .font_handles
                    .insert(font_name.clone(), font_handle);
            }

            // Load the script handles
            for script_relative_path in &meta.scripts {
                let (script_path, script_handle) =
                    get_relative_asset(load_context, &self_path, script_relative_path);
                dependencies.push(script_path);
                meta.script_handles.push(script_handle);
            }

            load_context.set_default_asset(LoadedAsset::new(meta).with_dependencies(dependencies));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["game.yml", "game.yaml"]
    }
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "421c9e38-89be-43ff-a293-6fea65abf946"]
pub struct EguiFont(pub egui::FontData);

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
