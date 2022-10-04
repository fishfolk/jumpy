use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use bevy::{
    asset::{Asset, AssetLoader, AssetPath, LoadedAsset},
    reflect::TypeUuid,
};
use bevy_egui::egui;
use bevy_mod_js_scripting::serde_json;

use crate::{
    metadata::{BorderImageMeta, GameMeta, MapLayerKind, MapMeta, PlayerMeta},
    prelude::*,
};

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<GameMeta>()
            .add_asset_loader(GameMetaLoader)
            .add_asset::<PlayerMeta>()
            .add_asset_loader(PlayerMetaLoader)
            .add_asset::<MapMeta>()
            .add_asset_loader(MapMetaLoader)
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
            let self_path = load_context.path();
            let mut meta: GameMeta = if self_path.extension() == Some(OsStr::new("json")) {
                serde_json::from_slice(bytes)?
            } else {
                serde_yaml::from_slice(bytes)?
            };
            trace!(path=?self_path, ?meta, "Loaded game asset");

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

            let mut dependencies = Vec::new();

            // Get locale handles
            for locale in &meta.translations.locales {
                let (path, handle) = get_relative_asset(load_context, self_path, locale);
                dependencies.push(path);
                meta.translations.locale_handles.push(handle);
            }

            // Load the main menu background
            let (main_menu_background_path, main_menu_background) = get_relative_asset(
                load_context,
                self_path,
                &meta.main_menu.background_image.image,
            );
            meta.main_menu.background_image.image_handle = main_menu_background;
            dependencies.push(main_menu_background_path);

            // Load UI border images
            let mut load_border_image = |border: &mut BorderImageMeta| {
                let (path, handle) = get_relative_asset(load_context, self_path, &border.image);
                dependencies.push(path);
                border.handle = handle;
            };
            load_border_image(&mut meta.ui_theme.hud.portrait_frame);
            load_border_image(&mut meta.ui_theme.panel.border);
            load_border_image(&mut meta.ui_theme.hud.lifebar.background_image);
            load_border_image(&mut meta.ui_theme.hud.lifebar.progress_image);
            for button in meta.ui_theme.button_styles.as_list() {
                load_border_image(&mut button.borders.default);
                if let Some(border) = &mut button.borders.clicked {
                    load_border_image(border);
                }
                if let Some(border) = &mut button.borders.focused {
                    load_border_image(border);
                }
            }

            // Load editor icons
            for icon in meta.ui_theme.editor.icons.as_mut_list() {
                let (path, handle) = get_relative_asset(load_context, self_path, &icon.image);
                icon.image_handle = handle;
                dependencies.push(path);
            }

            // Load player handles
            for player_relative_path in &meta.players {
                let (path, handle) =
                    get_relative_asset(load_context, self_path, player_relative_path);

                meta.player_handles.push(handle);
                dependencies.push(path);
            }

            // Load map handles
            for map_relative_path in &meta.maps {
                let (path, handle) = get_relative_asset(load_context, self_path, map_relative_path);

                meta.map_handles.push(handle);
                dependencies.push(path);
            }

            // Load UI fonts
            for (font_name, font_relative_path) in &meta.ui_theme.font_families {
                let (font_path, font_handle) =
                    get_relative_asset(load_context, self_path, font_relative_path);

                dependencies.push(font_path);

                meta.ui_theme
                    .font_handles
                    .insert(font_name.clone(), font_handle);
            }

            // Load the script handles
            for script_relative_path in &meta.scripts {
                let (script_path, script_handle) =
                    get_relative_asset(load_context, self_path, script_relative_path);
                dependencies.push(script_path);
                meta.script_handles.push(script_handle);
            }

            load_context.set_default_asset(LoadedAsset::new(meta).with_dependencies(dependencies));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["game.yml", "game.yaml", "game.json"]
    }
}

pub struct PlayerMetaLoader;

impl AssetLoader for PlayerMetaLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let self_path = load_context.path();
            let mut meta: PlayerMeta = if self_path.extension() == Some(OsStr::new("json")) {
                serde_json::from_slice(bytes)?
            } else {
                serde_yaml::from_slice(bytes)?
            };
            trace!(?self_path, ?meta, "Loaded player asset");

            let (atlas_path, atlas_handle) =
                get_relative_asset(load_context, load_context.path(), &meta.spritesheet.image);

            let atlas_handle = load_context.set_labeled_asset(
                "atlas",
                LoadedAsset::new(TextureAtlas::from_grid(
                    atlas_handle,
                    meta.spritesheet.tile_size.as_vec2(),
                    meta.spritesheet.columns,
                    meta.spritesheet.rows,
                ))
                .with_dependency(atlas_path),
            );
            meta.spritesheet.atlas_handle = atlas_handle;

            load_context.set_default_asset(LoadedAsset::new(meta));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["player.yml", "player.yaml", "player.json"]
    }
}

pub struct MapMetaLoader;

impl AssetLoader for MapMetaLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let self_path = load_context.path();
            let mut meta: MapMeta = if self_path.extension() == Some(OsStr::new("json")) {
                serde_json::from_slice(bytes)?
            } else {
                serde_yaml::from_slice(bytes)?
            };
            trace!(?self_path, ?meta, "Loaded map asset");

            let mut dependencies = Vec::new();

            // Load tile layer tilemaps
            for layer in &mut meta.layers {
                if let MapLayerKind::Tile(tile_layer) = &mut layer.kind {
                    let (path, handle) =
                        get_relative_asset(load_context, self_path, &tile_layer.tilemap);
                    tile_layer.tilemap_handle = handle;
                    dependencies.push(path);
                }
            }

            for layer in &mut meta.background_layers {
                let (path, handle) = get_relative_asset(load_context, self_path, &layer.image);
                dependencies.push(path);
                layer.image_handle = handle;

                // Rewrite relative paths from the parallax background layer as an absolute path to
                // make it compatible with the parallax plugin.
                layer.image = relative_asset_path(self_path, &layer.image)
                    .to_str()
                    .unwrap()
                    .into();
            }

            load_context.set_default_asset(LoadedAsset::new(meta).with_dependencies(dependencies));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["map.yml", "map.yaml", "map.json"]
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
