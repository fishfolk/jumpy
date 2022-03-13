use std::{collections::HashMap, fs, path::Path};
use std::any::{Any, TypeId};
use std::borrow::BorrowMut;
use std::hash::Hash;
use std::slice::Iter;
use std::collections::hash_map::Iter as HashMapIter;

use crate::particles::EmitterConfig;
use crate::text::Font;

use serde::{Deserialize, Serialize};

use serde::de::DeserializeOwned;

use crate::prelude::*;
use crate::audio::load_sound_file;
use crate::parsing::{deserialize_json_bytes, load_json_file};
use crate::error::{Result, ErrorKind};
use crate::texture::{Texture2D, TextureFilterMode, load_texture_file};
use crate::map::DecorationMetadata;
use crate::formaterr;
use crate::gui::theme::GuiTheme;

use crate::map::Map;

const PARTICLE_EFFECTS_DIR: &str = "particle_effects";
const SOUNDS_FILE: &str = "sounds";
const MUSIC_FILE: &str = "music";
const TEXTURES_FILE: &str = "textures";
const IMAGES_FILE: &str = "images";
const MAPS_FILE: &str = "maps";
const DECORATION_FILE: &str = "decoration";
const ITEMS_FILE: &str = "items";
const CHARACTERS_FILE: &str = "characters";

const DEFAULT_RESOURCE_FILES_EXTENSION: &str = "json";

pub const MAP_EXPORTS_DEFAULT_DIR: &str = "maps";
pub const MAP_EXPORTS_EXTENSION: &str = "json";
pub const MAP_EXPORT_NAME_MIN_LEN: usize = 1;

pub const MAP_PREVIEW_PLACEHOLDER_PATH: &str = "maps/no_preview.png";
pub const MAP_PREVIEW_PLACEHOLDER_ID: &str = "map_preview_placeholder";

const ACTIVE_MODS_FILE_NAME: &str = "active_mods";
const MOD_FILE_NAME: &str = "fishfight_mod";

static mut ASSETS_DIR: Option<String> = None;

pub fn assets_dir() -> &'static str {
    unsafe { ASSETS_DIR.as_ref().unwrap() }
}

static mut MODS_DIR: Option<String> = None;

pub fn mods_dir() -> &'static str {
    unsafe { MODS_DIR.as_ref().unwrap() }
}

static mut LOADED_MODS: Vec<ModMetadata> = Vec::new();

pub fn loaded_mods() -> Vec<ModMetadata> {
    unsafe { LOADED_MODS.clone() }
}

static mut PARTICLE_EFFECTS: Option<HashMap<String, EmitterConfig>> = None;

pub fn try_get_particle_effect(id: &str) -> Option<&EmitterConfig> {
    unsafe { PARTICLE_EFFECTS
        .as_ref()
        .unwrap_or_else(|| panic!("Attempted to load a particle effect resource but resources has not been initialized"))
        .get(id) }
}

pub fn get_particle_effect(id: &str) -> &EmitterConfig {
    try_get_particle_effect(id).unwrap()
}

pub fn iter_particle_effects() -> HashMapIter<'static, String, EmitterConfig> {
    unsafe { PARTICLE_EFFECTS
        .as_ref()
        .unwrap_or_else(|| panic!("Attempted to iter particle effect resources but resources has not been initialized"))
        .iter() }
}

static mut SOUNDS: Option<HashMap<String, Sound>> = None;

pub fn try_get_sound(id: &str) -> Option<&mut Sound> {
    unsafe { SOUNDS
        .as_mut()
        .unwrap_or_else(|| panic!("Attempted to load a sound resource but resources has not been initialized"))
        .get_mut(id) }
}

pub fn get_sound(id: &str) -> &mut Sound {
    try_get_sound(id).unwrap()
}

static mut MUSIC: Option<HashMap<String, Sound>> = None;

pub fn try_get_music(id: &str) -> Option<&mut Sound> {
    unsafe { MUSIC
        .as_mut()
        .unwrap_or_else(|| panic!("Attempted to load a music resource but resources has not been initialized"))
        .get_mut(id) }
}

pub fn get_music(id: &str) -> &mut Sound {
    try_get_music(id).unwrap()
}

static mut TEXTURES: Option<HashMap<String, TextureResource>> = None;

pub fn try_get_texture(id: &str) -> Option<&TextureResource> {
    unsafe { TEXTURES
        .as_ref()
        .unwrap_or_else(|| panic!("Attempted to load a texture resource but resources has not been initialized"))
        .get(id) }
}

pub fn get_texture(id: &str) -> &TextureResource {
    try_get_texture(id).unwrap()
}

pub fn iter_textures() -> HashMapIter<'static, String, TextureResource> {
    unsafe { TEXTURES
        .as_ref()
        .unwrap_or_else(|| panic!("Attempted to iter texture resources but resources has not been initialized"))
        .iter() }
}

static mut FONTS: Option<HashMap<String, Font>> = None;

pub fn try_get_font(id: &str) -> Option<&Font> {
    unsafe { FONTS
        .as_ref()
        .unwrap_or_else(|| panic!("Attempted to load a font resource but resources has not been initialized"))
        .get(id) }
}

pub fn get_font(id: &str) -> &Font {
    try_get_font(id).unwrap()
}

static mut MAPS: Vec<MapResource> = Vec::new();

pub fn iter_maps() -> Iter<'static, MapResource> {
    unsafe { MAPS.iter() }
}

pub fn try_get_map(index: usize) -> Option<&'static MapResource> {
    unsafe { MAPS.get(index) }
}

pub fn get_map(index: usize) -> &'static MapResource {
    try_get_map(index).unwrap()
}

static mut DECORATION: Option<HashMap<String, DecorationMetadata>> = None;

pub fn try_get_decoration(id: &str) -> Option<&DecorationMetadata> {
    unsafe { DECORATION
        .as_ref()
        .unwrap_or_else(|| panic!("Attempted to load a decoration resource but resources has not been initialized"))
        .get(id) }
}

pub fn get_decoration(id: &str) -> &DecorationMetadata {
    try_get_decoration(id).unwrap()
}

pub fn iter_decoration() -> HashMapIter<'static, String, DecorationMetadata> {
    unsafe { DECORATION
        .as_ref()
        .unwrap_or_else(|| panic!("Attempted to iter decoration resources but resources has not been initialized"))
        .iter() }
}

#[cfg(feature = "macroquad-backend")]
static mut IMAGES: Option<HashMap<String, ImageResource>> = None;

#[cfg(feature = "macroquad-backend")]
pub fn try_get_image(id: &str) -> Option<&ImageResource> {
    unsafe { IMAGES
        .as_ref()
        .unwrap_or_else(|| panic!("Attempted to load an image resource but resources has not been initialized"))
        .get(id) }
}

#[cfg(feature = "macroquad-backend")]
pub fn get_image(id: &str) -> &ImageResource {
    try_get_image(id).unwrap()
}

#[cfg(feature = "macroquad-backend")]
pub fn iter_images() -> HashMapIter<'static, String, ImageResource> {
    unsafe { IMAGES
        .as_ref()
        .unwrap_or_else(|| panic!("Attempted to iter image resources but resources has not been initialized"))
        .iter() }
}

// TODO: Cleaner handling of custom resources
pub async fn load_resources<P, C, I>(path: P, characters: &mut Vec<C>, items: &mut HashMap<String, I>, should_overwrite: bool) -> Result<()>
    where P: AsRef<Path>, C: Clone + DeserializeOwned, I: Clone + CustomResource + DeserializeOwned {
    let path = path.as_ref();

    unsafe { ASSETS_DIR = Some(path.to_string_lossy().to_string()) };

    load_particle_effects(&path,  should_overwrite).await?;
    load_sounds(&path,  should_overwrite).await?;
    load_music(&path,  should_overwrite).await?;
    load_textures(&path,  should_overwrite).await?;
    load_decoration(&path,  should_overwrite).await?;
    load_maps(&path,  should_overwrite).await?;

    #[cfg(feature = "macroquad-backend")]
    load_images(&path,  should_overwrite).await?;

    load_characters(&path, characters, should_overwrite).await?;
    load_items(&path, items, should_overwrite).await?;

    Ok(())
}

pub async fn load_mods<P, C, I>(path: P, characters: &mut Vec<C>, items: &mut HashMap<String, I>) -> Result<()>
    where P: AsRef<Path>, C: Clone + DeserializeOwned, I: Clone + CustomResource + DeserializeOwned {
    let path = path.as_ref();

    unsafe { MODS_DIR = Some(path.to_string_lossy().to_string()) };

    let active_mods_file_path = path
        .join(ACTIVE_MODS_FILE_NAME)
        .with_extension(DEFAULT_RESOURCE_FILES_EXTENSION);

    let loaded_mods: &mut Vec<ModMetadata> = unsafe { LOADED_MODS.as_mut() };

    let mod_dirs: Vec<String> = load_json_file(active_mods_file_path).await?;

    for mod_dir in mod_dirs.iter() {
        let mod_dir_path = path.join(mod_dir);

        let mod_file_path = mod_dir_path
            .join(MOD_FILE_NAME)
            .with_extension(DEFAULT_RESOURCE_FILES_EXTENSION);

        let meta: ModMetadata = load_json_file(mod_file_path).await?;

        let mut has_game_version_mismatch = false;

        if let Some(req_version) = &meta.game_version {
            if *req_version != env!("CARGO_PKG_VERSION") {
                has_game_version_mismatch = true;

                #[cfg(debug_assertions)]
                println!(
                    "Loading mod {} (v{}) failed: Game version requirement mismatch (v{})",
                    &meta.id, &meta.version, req_version
                );
            }
        }

        if !has_game_version_mismatch {
            let mut has_unmet_dependencies = false;

            for dependency in &meta.dependencies {
                let res = loaded_mods
                    .iter()
                    .find(|&meta| meta.id == dependency.id && meta.version == dependency.version);

                if res.is_none() {
                    has_unmet_dependencies = true;

                    #[cfg(debug_assertions)]
                    println!(
                        "Loading mod {} (v{}) failed: Unmet dependency {} (v{})",
                        &meta.id, &meta.version, &dependency.id, &dependency.version
                    );

                    break;
                }
            }

            if !has_unmet_dependencies {
                load_resources(mod_dir_path, characters, items, false).await?;

                #[cfg(debug_assertions)]
                println!("Loaded mod {} (v{})", &meta.id, &meta.version);

                loaded_mods.push(meta);
            }
        }
    }

    Ok(())
}

async fn load_particle_effects<P: AsRef<Path>>(path: P, should_overwrite: bool) -> Result<()> {
    if unsafe { PARTICLE_EFFECTS.is_none() } {
        unsafe { PARTICLE_EFFECTS = Some(HashMap::new()) }
    }

    let particle_effects = unsafe { PARTICLE_EFFECTS
        .as_mut()
        .unwrap() };

    if should_overwrite {
        particle_effects.clear();
    }

    let particle_effects_file_path = path
        .as_ref()
        .join(PARTICLE_EFFECTS_DIR)
        .with_extension(DEFAULT_RESOURCE_FILES_EXTENSION);

    if let Ok(bytes) = load_file(&particle_effects_file_path).await {
        let metadata: Vec<ParticleEffectMetadata> = deserialize_json_bytes(&bytes)?;

        for meta in metadata {
            let file_path = path.as_ref().join(&meta.path);

            let cfg: EmitterConfig = load_json_file(&file_path).await?;

            particle_effects.insert(meta.id, cfg);
        }
    }

    Ok(())
}

async fn load_sounds<P: AsRef<Path>>(path: P, should_overwrite: bool) -> Result<()> {
    if unsafe { SOUNDS.is_none() } {
        unsafe { SOUNDS = Some(HashMap::new()) }
    }

    let sounds = unsafe { SOUNDS.as_mut().unwrap() };

    if should_overwrite {
        sounds.clear();
    }

    let sounds_file_path = path
        .as_ref()
        .join(SOUNDS_FILE)
        .with_extension(DEFAULT_RESOURCE_FILES_EXTENSION);

    if let Ok(bytes) = load_file(&sounds_file_path).await {
        let metadata: Vec<SoundMetadata> = deserialize_json_bytes(&bytes)?;

        for meta in metadata {
            let file_path = path.as_ref().join(meta.path);

            let sound = load_sound_file(&file_path).await?;

            sounds.insert(meta.id, sound);
        }
    }

    Ok(())
}

async fn load_music<P: AsRef<Path>>(path: P, should_overwrite: bool) -> Result<()> {
    if unsafe { MUSIC.is_none() } {
        unsafe { MUSIC = Some(HashMap::new()) }
    }

    let music = unsafe { MUSIC.as_mut().unwrap() };

    if should_overwrite {
        music.clear();
    }

    let music_file_path = path
        .as_ref()
        .join(MUSIC_FILE)
        .with_extension(DEFAULT_RESOURCE_FILES_EXTENSION);

    if let Ok(bytes) = load_file(&music_file_path).await {
        let metadata: Vec<SoundMetadata> = deserialize_json_bytes(&bytes)?;

        for meta in metadata {
            let file_path = path.as_ref().join(meta.path);

            let sound = load_sound_file(&file_path).await?;

            music.insert(meta.id, sound);
        }
    }

    Ok(())
}

async fn load_textures<P: AsRef<Path>>(path: P, should_overwrite: bool) -> Result<()> {
    if unsafe { TEXTURES.is_none() } {
        unsafe { TEXTURES = Some(HashMap::new()) }
    }

    let textures = unsafe { TEXTURES.as_mut().unwrap() };

    if should_overwrite {
        textures.clear();
    }

    let textures_file_path = path
        .as_ref()
        .join(TEXTURES_FILE)
        .with_extension(DEFAULT_RESOURCE_FILES_EXTENSION);

    if let Ok(bytes) = load_file(&textures_file_path).await {
        let metadata: Vec<TextureMetadata> = deserialize_json_bytes(&bytes)?;

        for meta in metadata {
            let file_path = path.as_ref().join(&meta.path);

            let texture = load_texture_file(&file_path, meta.format, meta.filter_mode).await?;

            let key = meta.id.clone();

            #[cfg(debug_assertions)]
            if meta.frame_size.is_none()
                && meta.kind.is_some()
                && meta.kind.unwrap() == TextureKind::Spritesheet
            {
                println!(
                    "WARNING: The texture '{}' is a spritesheet but no frame size has been set",
                    &meta.id
                );
            }

            let res = TextureResource {
                texture,
                meta,
            };

            textures.insert(key, res);
        }
    }

    Ok(())
}

async fn load_maps<P: AsRef<Path>>(path: P, should_overwrite: bool) -> Result<()> {
    let maps: &mut Vec<MapResource> = unsafe { MAPS.borrow_mut() };

    if should_overwrite {
        maps.clear();
    }

    let maps_file_path = path
        .as_ref()
        .join(MAPS_FILE)
        .with_extension(DEFAULT_RESOURCE_FILES_EXTENSION);

    if let Ok(bytes) = load_file(&maps_file_path).await {
        let metadata: Vec<MapMetadata> = deserialize_json_bytes(&bytes)?;

        for meta in metadata {
            let map_path = path.as_ref().join(&meta.path);
            let preview_path = path.as_ref().join(&meta.preview_path);

            let map = if meta.is_tiled_map {
                Map::load_tiled(map_path, None).await?
            } else {
                Map::load(map_path).await?
            };

            let preview = load_texture_file(&preview_path, TextureFormat::Png, TextureFilterMode::Nearest).await?;

            let res = MapResource { map, preview, meta };

            maps.push(res)
        }
    }

    Ok(())
}

async fn load_decoration<P: AsRef<Path>>(path: P, should_overwrite: bool) -> Result<()> {
    if unsafe { DECORATION.is_none() } {
        unsafe { DECORATION = Some(HashMap::new()) }
    }

    let decoration = unsafe { DECORATION.as_mut().unwrap() };

    if should_overwrite {
        decoration.clear();
    }

    let decoration_file_path = path
        .as_ref()
        .join(DECORATION_FILE)
        .with_extension(DEFAULT_RESOURCE_FILES_EXTENSION);

    if let Ok(bytes) = load_file(&decoration_file_path).await {
        let decoration_paths: Vec<String> = deserialize_json_bytes(&bytes)?;

        for decoration_path in decoration_paths {
            let path = path.as_ref().join(&decoration_path);

            let params: DecorationMetadata = load_json_file(&path).await?;

            decoration.insert(params.id.clone(), params);
        }
    }

    Ok(())
}

#[cfg(feature = "macroquad-backend")]
async fn load_images<P: AsRef<Path>>(path: P, should_overwrite: bool) -> Result<()> {
    if unsafe { IMAGES.is_none() } {
        unsafe { IMAGES = Some(HashMap::new()) }
    }

    let images = unsafe { IMAGES.as_mut().unwrap() };

    if should_overwrite {
        images.clear();
    }

    let images_file_path = path
        .as_ref()
        .join(IMAGES_FILE)
        .with_extension(DEFAULT_RESOURCE_FILES_EXTENSION);

    if let Ok(bytes) = load_file(&images_file_path).await {
        let metadata: Vec<ImageMetadata> = deserialize_json_bytes(&bytes)?;

        for meta in metadata {
            let file_path = path.as_ref().join(&meta.path);

            let image = crate::macroquad::texture::load_image(&file_path.to_string_lossy()).await?;

            let key = meta.id.clone();

            let meta = ImageMetadata {
                size: vec2(image.width() as f32, image.height() as f32),
                ..meta
            };

            let res = ImageResource { image, meta };

            images.insert(key, res);
        }
    }

    Ok(())
}

async fn load_characters<P: AsRef<Path>, C: Clone + DeserializeOwned>(path: P, characters: &mut Vec<C>, should_overwrite: bool) -> Result<()> {
    if should_overwrite {
        characters.clear();
    }

    let characters_file_path = path
        .as_ref()
        .join(CHARACTERS_FILE)
        .with_extension(DEFAULT_RESOURCE_FILES_EXTENSION);

    if let Ok(bytes) = load_file(&characters_file_path).await {
        let mut meta: Vec<C> = deserialize_json_bytes(&bytes)?;
        characters.append(&mut meta);
    }

    Ok(())
}

async fn load_items<P: AsRef<Path>, I: Clone + CustomResource + DeserializeOwned>(path: P, items: &mut HashMap<String, I>, should_overwrite: bool) -> Result<()> {
    if should_overwrite {
        items.clear();
    }

    let items_file_path = path
        .as_ref()
        .join(ITEMS_FILE)
        .with_extension(DEFAULT_RESOURCE_FILES_EXTENSION);

    if let Ok(bytes) = load_file(&items_file_path).await {
        let items_paths: Vec<String> = deserialize_json_bytes(&bytes)?;

        for item_path in items_paths {
            let path = path.as_ref().join(&item_path);

            let params: I = load_json_file(&path).await?;

            items.insert(params.id(), params);
        }
    }

    Ok(())
}

pub trait CustomResource {
    fn id(&self) -> String;
}

#[derive(Serialize, Deserialize)]
struct ParticleEffectMetadata {
    id: String,
    path: String,
}

#[derive(Serialize, Deserialize)]
struct SoundMetadata {
    id: String,
    path: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureKind {
    Background,
    Tileset,
    Spritesheet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureMetadata {
    pub id: String,
    pub path: String,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<TextureKind>,
    #[serde(
        default,
        alias = "sprite_size",
        skip_serializing_if = "Option::is_none"
    )]
    pub frame_size: Option<Size<f32>>,
    #[serde(default = "TextureFilterMode::default")]
    pub filter_mode: TextureFilterMode,
    #[serde(default = "TextureFormat::default")]
    pub format: TextureFormat,
}

#[derive(Debug, Clone)]
pub struct TextureResource {
    pub texture: Texture2D,
    pub meta: TextureMetadata,
}

impl TextureResource {
    pub fn frame_size(&self) -> Size<f32> {
        self.meta.frame_size.unwrap_or_else(|| self.texture.size())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub id: String,
    pub path: String,
    #[serde(default, skip)]
    pub size: Vec2,
}

#[cfg(feature = "macroquad-backend")]
#[derive(Debug, Clone)]
pub struct ImageResource {
    pub image: crate::macroquad::texture::Image,
    pub meta: ImageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMetadata {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub path: String,
    pub preview_path: String,
    #[serde(default, skip_serializing_if = "crate::json::is_false")]
    pub is_tiled_map: bool,
    #[serde(default, skip_serializing_if = "crate::json::is_false")]
    pub is_user_map: bool,
}

#[derive(Debug, Clone)]
pub struct MapResource {
    pub map: Map,
    pub preview: Texture2D,
    pub meta: MapMetadata,
}

pub fn create_map(
    name: &str,
    description: Option<&str>,
    tile_size: Vec2,
    grid_size: UVec2,
) -> Result<MapResource> {
    let description = description.map(|str| str.to_string());

    let map_path = Path::new(MAP_EXPORTS_DEFAULT_DIR)
        .join(map_name_to_filename(name))
        .with_extension(MAP_EXPORTS_EXTENSION);

    let preview_path = Path::new(MAP_PREVIEW_PLACEHOLDER_PATH);

    let meta = MapMetadata {
        name: name.to_string(),
        description,
        path: map_path.to_string_lossy().to_string(),
        preview_path: preview_path.to_string_lossy().to_string(),
        is_tiled_map: false,
        is_user_map: true,
    };

    let map = Map::new(tile_size, grid_size);

    let preview = {
        let res = get_texture(MAP_PREVIEW_PLACEHOLDER_ID);
        res.texture
    };

    Ok(MapResource { map, preview, meta })
}

pub fn save_map(map_resource: &MapResource) -> Result<()> {
    let assets_path = Path::new(assets_dir());
    let export_path = assets_path.join(&map_resource.meta.path);

    {
        let maps: &mut Vec<MapResource> = unsafe { MAPS.borrow_mut() };

        if export_path.exists() {
            let mut i = 0;
            while i < maps.len() {
                let res = &maps[i];
                if res.meta.path == map_resource.meta.path {
                    if res.meta.is_user_map {
                        maps.remove(i);
                        break;
                    } else {
                        return Err(formaterr!(
                                ErrorKind::General,
                                "Resources: The path '{}' is in use and it is not possible to overwrite. Please choose a different map name",
                                &map_resource.meta.path,
                            ));
                    }
                }

                i += 1;
            }
        }

        map_resource.map.save(export_path)?;

        maps.push(map_resource.clone());
    }

    save_maps_file()?;

    Ok(())
}

pub fn delete_map(index: usize) -> Result<()> {
    let map_resource = unsafe { MAPS.remove(index) };

    let path = Path::new(assets_dir()).join(&map_resource.meta.path);

    fs::remove_file(path)?;

    save_maps_file()?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn save_maps_file() -> Result<()> {
    let maps_file_path = Path::new(assets_dir())
        .join(MAPS_FILE)
        .with_extension(DEFAULT_RESOURCE_FILES_EXTENSION);

    let metadata: Vec<MapMetadata> = iter_maps().map(|res| res.meta.clone()).collect();

    let str = serde_json::to_string_pretty(&metadata)?;
    fs::write(maps_file_path, &str)?;

    Ok(())
}

pub fn is_valid_map_export_path<P: AsRef<Path>>(path: P, should_overwrite: bool) -> bool {
    let path = path.as_ref();

    if let Some(file_name) = path.file_name() {
        if is_valid_map_file_name(&file_name.to_string_lossy().to_string()) {
            let res = iter_maps()
                .find(|res| Path::new(&res.meta.path) == path);

            if let Some(res) = res {
                return res.meta.is_user_map && should_overwrite;
            }

            return true;
        }
    }

    false
}

pub fn map_name_to_filename(name: &str) -> String {
    name.replace(' ', "_").replace('.', "_").to_lowercase()
}

pub fn is_valid_map_file_name(file_name: &str) -> bool {
    if file_name.len() - MAP_EXPORTS_EXTENSION.len() > MAP_EXPORT_NAME_MIN_LEN {
        if let Some(extension) = Path::new(file_name).extension() {
            return extension == MAP_EXPORTS_EXTENSION;
        }
    }

    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub kind: ModKind,
    #[serde(default)]
    pub dependencies: Vec<DependencyMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyMetadata {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModKind {
    DataOnly,
    Full,
}

impl Default for ModKind {
    fn default() -> Self {
        ModKind::Full
    }
}