use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::utils::path::NormalizePath;
use bevy::{
    asset::{Asset, AssetLoader, AssetPath, LoadedAsset},
    reflect::TypeUuid,
    render::texture::ImageTextureLoader,
};
use bevy_egui::egui;
use bevy_mod_js_scripting::{serde_json, JsScript};

use crate::{
    config::ENGINE_CONFIG,
    metadata::{
        BorderImageMeta, GameMeta, MapElementMeta, MapLayerKind, MapMeta, PlayerMeta,
        TextureAtlasMeta,
    },
    prelude::*,
};

mod asset_handle;
pub use asset_handle::AssetHandle;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AssetHandle<JsScript>>()
            .register_type::<AssetHandle<Image>>()
            .add_jumpy_asset::<GameMeta>()
            .add_asset_loader(GameMetaLoader)
            .add_jumpy_asset::<PlayerMeta>()
            .add_asset_loader(PlayerMetaLoader)
            .add_jumpy_asset::<MapMeta>()
            .add_asset_loader(MapMetaLoader)
            .add_jumpy_asset::<MapElementMeta>()
            .add_asset_loader(MapElementMetaLoader)
            .add_asset_loader(TextureAtlasLoader)
            .add_jumpy_asset::<EguiFont>()
            .add_asset_loader(EguiFontLoader);

        if ENGINE_CONFIG.server_mode {
            let image_loader = ImageTextureLoader::from_world(&mut app.world);
            app.add_asset::<Image>().add_asset_loader(image_loader);
        }
    }
}

trait AppExt {
    fn add_jumpy_asset<T: Asset>(&mut self) -> &mut Self;
}
impl AppExt for App {
    fn add_jumpy_asset<T: Asset>(&mut self) -> &mut Self {
        self.add_asset::<T>().register_type::<AssetHandle<T>>()
    }
}

/// Calculate an asset's full path relative to another asset
fn relative_asset_path(asset_path: &Path, relative_path: &str) -> PathBuf {
    let is_relative = !relative_path.starts_with('/');

    let path = if is_relative {
        let base = asset_path.parent().unwrap_or_else(|| Path::new(""));
        base.join(relative_path)
    } else {
        Path::new(relative_path)
            .strip_prefix("/")
            .unwrap()
            .to_owned()
    };
    path.normalize()
}

/// Helper to get relative asset paths and handles
fn get_relative_asset(
    load_context: &bevy::asset::LoadContext,
    self_path: &Path,
    relative_path: &str,
) -> (AssetPath<'static>, HandleUntyped) {
    let asset_path = relative_asset_path(self_path, relative_path);
    let asset_path = AssetPath::new(asset_path, None);
    // Note: Because the load context doesn't have a get_handle_untyped, we have to feed it a dummy
    // asset type, in this case GameMeta, and then subsequently clone it as untyped.
    let handle = load_context
        .get_handle::<_, GameMeta>(asset_path.clone())
        .clone_untyped();

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
                dependencies.push(path.clone());
                meta.translations
                    .locale_handles
                    .push(AssetHandle::new(path, handle.typed()));
            }

            // Load the main menu background
            let (main_menu_background_path, main_menu_background) = get_relative_asset(
                load_context,
                self_path,
                &meta.main_menu.background_image.image,
            );
            meta.main_menu.background_image.image_handle = AssetHandle::new(
                main_menu_background_path.clone(),
                main_menu_background.typed(),
            );
            dependencies.push(main_menu_background_path);

            // Load UI border images
            let mut load_border_image = |border: &mut BorderImageMeta| {
                let (path, handle) = get_relative_asset(load_context, self_path, &border.image);
                dependencies.push(path.clone());
                border.handle = AssetHandle::new(path, handle.typed());
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
                icon.image_handle = AssetHandle::new(path.clone(), handle.typed());
                dependencies.push(path);
            }

            // Load player handles
            for player_relative_path in &meta.players {
                let (path, handle) =
                    get_relative_asset(load_context, self_path, player_relative_path);

                meta.player_handles
                    .push(AssetHandle::new(path.clone(), handle.typed()));
                dependencies.push(path);
            }

            // Load map handles
            for map_relative_path in &meta.maps {
                let (path, handle) = get_relative_asset(load_context, self_path, map_relative_path);

                meta.map_handles
                    .push(AssetHandle::new(path.clone(), handle.typed()));
                dependencies.push(path);
            }

            // Load UI fonts
            for (font_name, font_relative_path) in &meta.ui_theme.font_families {
                let (font_path, font_handle) =
                    get_relative_asset(load_context, self_path, font_relative_path);

                dependencies.push(font_path.clone());

                meta.ui_theme.font_handles.insert(
                    font_name.clone(),
                    AssetHandle::new(font_path, font_handle.typed()),
                );
            }

            // Load the script handles
            for script_relative_path in &meta.scripts {
                let (script_path, script_handle) =
                    get_relative_asset(load_context, self_path, script_relative_path);
                dependencies.push(script_path.clone());
                meta.script_handles
                    .push(AssetHandle::new(script_path, script_handle.typed()));
            }

            // Load the client_script handles
            for script_relative_path in &meta.client_scripts {
                let (script_path, script_handle) =
                    get_relative_asset(load_context, self_path, script_relative_path);
                dependencies.push(script_path.clone());
                meta.client_script_handles
                    .push(AssetHandle::new(script_path, script_handle.typed()));
            }

            // Load the serer_script handles
            for script_relative_path in &meta.server_scripts {
                let (script_path, script_handle) =
                    get_relative_asset(load_context, self_path, script_relative_path);
                dependencies.push(script_path.clone());
                meta.server_script_handles
                    .push(AssetHandle::new(script_path, script_handle.typed()));
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
                    atlas_handle.typed(),
                    meta.spritesheet.tile_size.as_vec2(),
                    meta.spritesheet.columns,
                    meta.spritesheet.rows,
                ))
                .with_dependency(atlas_path.clone()),
            );
            meta.spritesheet.atlas_handle = AssetHandle::new(atlas_path, atlas_handle);

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

            // Load layer elements and tilemaps
            for layer in &mut meta.layers {
                match &mut layer.kind {
                    MapLayerKind::Tile(tile_layer) => {
                        let (path, handle) =
                            get_relative_asset(load_context, self_path, &tile_layer.tilemap);
                        tile_layer.tilemap_handle = AssetHandle::new(path.clone(), handle.typed());
                        dependencies.push(path);
                    }
                    MapLayerKind::Element(element_layer) => {
                        for element in &mut element_layer.elements {
                            let (path, handle) =
                                get_relative_asset(load_context, self_path, &element.element);
                            element.element_handle = AssetHandle::new(path.clone(), handle.typed());
                            dependencies.push(path);
                        }
                    }
                }
            }

            // Load parallax background layers
            for layer in &mut meta.background_layers {
                let (path, handle) = get_relative_asset(load_context, self_path, &layer.image);
                dependencies.push(path.clone());
                layer.image_handle = AssetHandle::new(path, handle.typed());

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

pub struct MapElementMetaLoader;

impl AssetLoader for MapElementMetaLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let self_path = load_context.path();
            let mut meta: MapElementMeta = if self_path.extension() == Some(OsStr::new("json")) {
                serde_json::from_slice(bytes)?
            } else {
                serde_yaml::from_slice(bytes)?
            };
            trace!(?self_path, ?meta, "Loaded map element asset");

            let mut dependencies = Vec::new();

            // Load the element script
            for script in &meta.scripts {
                let (script_path, script_handle) =
                    get_relative_asset(load_context, self_path, script);
                meta.script_handles
                    .push(AssetHandle::new(script_path.clone(), script_handle.typed()));
                dependencies.push(script_path);
            }

            // Load preloaded assets
            for asset in &meta.preload_assets {
                let (path, handle) = get_relative_asset(load_context, self_path, asset);
                dependencies.push(path);
                meta.preload_asset_handles.push(handle);
            }

            load_context.set_default_asset(LoadedAsset::new(meta).with_dependencies(dependencies));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["element.yml", "element.yaml", "element.json"]
    }
}

pub struct TextureAtlasLoader;

impl AssetLoader for TextureAtlasLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            let self_path = load_context.path();
            let meta: TextureAtlasMeta = if self_path.extension() == Some(OsStr::new("json")) {
                serde_json::from_slice(bytes)?
            } else {
                serde_yaml::from_slice(bytes)?
            };
            trace!(?self_path, ?meta, "Loaded texture atlas asset");

            let (image_path, image_handle) =
                get_relative_asset(load_context, self_path, &meta.image);

            let atlas = TextureAtlas::from_grid_with_padding(
                image_handle.typed(),
                meta.tile_size,
                meta.columns,
                meta.rows,
                meta.padding,
                meta.offset,
            );

            load_context.set_default_asset(LoadedAsset::new(atlas).with_dependency(image_path));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["atlas.yml", "atlas.yaml", "atlas.json"]
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
