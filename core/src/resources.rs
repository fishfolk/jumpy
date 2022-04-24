use std::any::{Any, TypeId};
use std::borrow::BorrowMut;
use std::collections::hash_map::Iter as HashMapIter;
use std::future::Future;
use std::hash::Hash;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::pin::Pin;
use std::slice::{Iter, IterMut};
use std::{collections::HashMap, fs, path::Path};

use async_trait::async_trait;

use crate::particles::EmitterConfig;
use crate::text::{load_font, Font};

use serde::{Deserialize, Serialize};

use serde::de::DeserializeOwned;

use crate::audio::load_sound_file;
use crate::error::{ErrorKind, Result};
use crate::formaterr;
use crate::gui::rebuild_gui_theme;
use crate::map::DecorationMetadata;
use crate::parsing::{deserialize_json_bytes, load_json_file};
use crate::prelude::*;
use crate::texture::{load_texture_file, Texture2D, TextureFilterMode};

use crate::map::Map;

const PARTICLE_EFFECTS_FILE: &str = "particle_effects";
const AUDIO_FILE: &str = "audio";
const MUSIC_FILE: &str = "music";
const TEXTURES_FILE: &str = "textures";
const IMAGES_FILE: &str = "images";
const FONTS_FILE: &str = "fonts";
const MAPS_FILE: &str = "maps";
const DECORATION_FILE: &str = "decoration";

pub const DEFAULT_RESOURCE_FILE_EXTENSION: &str = "json";

pub const MAP_EXPORTS_DEFAULT_DIR: &str = "maps";
pub const MAP_EXPORTS_EXTENSION: &str = "json";
pub const MAP_EXPORT_NAME_MIN_LEN: usize = 1;

pub const MAP_PREVIEW_PLACEHOLDER_PATH: &str = "maps/no_preview.png";
pub const MAP_PREVIEW_PLACEHOLDER_ID: &str = "map_preview_placeholder";

const ACTIVE_MODS_FILE_NAME: &str = "active_mods";
const MOD_FILE_NAME: &str = "fishfight_mod";

pub trait Resource: Clone + DeserializeOwned {}

pub trait ResourceId: Resource {
    fn id(&self) -> String;
}

#[async_trait]
pub trait ResourceMap: ResourceId {
    fn storage() -> &'static HashMap<String, Self>;

    async fn load<P: AsRef<Path> + Send, E: Into<Option<&'static str>> + Send>(
        path: P,
        ext: E,
        is_required: bool,
        should_overwrite: bool,
    ) -> Result<()>;

    fn iter() -> std::collections::hash_map::Iter<'static, String, Self> {
        Self::storage().iter()
    }

    fn try_get(id: &str) -> Option<&'static Self> {
        Self::storage().get(id)
    }

    fn get(id: &str) -> &'static Self {
        Self::try_get(id).unwrap()
    }
}

pub trait ResourceMapMut: ResourceMap {
    fn storage_mut() -> &'static mut HashMap<String, Self>;

    fn iter_mut() -> std::collections::hash_map::IterMut<'static, String, Self> {
        Self::storage_mut().iter_mut()
    }

    fn try_get_mut(id: &str) -> Option<&'static mut Self> {
        Self::storage_mut().get_mut(id)
    }

    fn get_mut(id: &str) -> &'static mut Self {
        Self::try_get_mut(id).unwrap()
    }
}

#[async_trait]
pub trait ResourceVec: Resource {
    fn storage() -> &'static Vec<Self>;

    async fn load<P: AsRef<Path> + Send, E: Into<Option<&'static str>> + Send>(
        path: P,
        ext: E,
        is_required: bool,
        should_overwrite: bool,
    ) -> Result<()>;

    fn iter() -> Iter<'static, Self> {
        Self::storage().iter()
    }

    fn try_get(index: usize) -> Option<&'static Self> {
        Self::storage().get(index)
    }

    fn get(index: usize) -> &'static Self {
        Self::try_get(index).unwrap()
    }
}

pub trait ResourceVecMut: ResourceVec {
    fn storage_mut() -> &'static mut Vec<Self>;

    fn iter_mut() -> IterMut<'static, Self> {
        Self::storage_mut().iter_mut()
    }

    fn try_get_mut(index: usize) -> Option<&'static mut Self> {
        Self::storage_mut().get_mut(index)
    }

    fn get_mut(index: usize) -> &'static mut Self {
        Self::try_get_mut(index).unwrap()
    }
}

const DEFAULT_ASSETS_DIR: &str = "assets/";

static mut ASSETS_DIR: Option<String> = None;

pub fn set_assets_dir<P: AsRef<Path>>(path: P) {
    let str = path.as_ref().to_string_lossy().to_string();
    unsafe {
        ASSETS_DIR = Some(str);
    }
}

pub fn assets_dir() -> String {
    unsafe {
        ASSETS_DIR
            .get_or_insert_with(|| DEFAULT_ASSETS_DIR.to_string())
            .clone()
    }
}

const DEFAULT_MODS_DIR: &str = "mods/";

static mut MODS_DIR: Option<String> = None;

pub fn set_mods_dir<P: AsRef<Path>>(path: P) {
    let str = path.as_ref().to_string_lossy().to_string();
    unsafe {
        MODS_DIR = Some(str);
    }
}

pub fn mods_dir() -> String {
    unsafe {
        MODS_DIR
            .get_or_insert_with(|| DEFAULT_MODS_DIR.to_string())
            .clone()
    }
}

static mut LOADED_MODS: Vec<ModMetadata> = Vec::new();

pub fn loaded_mods() -> &'static [ModMetadata] {
    unsafe { LOADED_MODS.as_slice() }
}

pub(crate) fn loaded_mods_mut() -> &'static mut Vec<ModMetadata> {
    unsafe { LOADED_MODS.as_mut() }
}

static mut PARTICLE_EFFECTS: Option<HashMap<String, EmitterConfig>> = None;

pub fn try_get_particle_effect(id: &str) -> Option<&EmitterConfig> {
    unsafe { PARTICLE_EFFECTS.get_or_insert_with(HashMap::new).get(id) }
}

pub fn get_particle_effect(id: &str) -> &EmitterConfig {
    try_get_particle_effect(id).unwrap()
}

pub fn iter_particle_effects() -> HashMapIter<'static, String, EmitterConfig> {
    unsafe { PARTICLE_EFFECTS.get_or_insert_with(HashMap::new).iter() }
}

static mut AUDIO: Option<HashMap<String, Sound>> = None;

pub fn try_get_sound(id: &str) -> Option<&Sound> {
    unsafe { AUDIO.get_or_insert_with(HashMap::new).get(id) }
}

pub fn get_sound(id: &str) -> &Sound {
    try_get_sound(id).unwrap()
}

static mut TEXTURES: Option<HashMap<String, TextureResource>> = None;

pub fn try_get_texture(id: &str) -> Option<&TextureResource> {
    unsafe { TEXTURES.get_or_insert_with(HashMap::new).get(id) }
}

pub fn get_texture(id: &str) -> &TextureResource {
    try_get_texture(id).unwrap()
}

pub fn iter_textures() -> HashMapIter<'static, String, TextureResource> {
    unsafe { TEXTURES.get_or_insert_with(HashMap::new).iter() }
}

static mut FONTS: Option<HashMap<String, Font>> = None;

pub fn try_get_font(id: &str) -> Option<Font> {
    unsafe { FONTS.get_or_insert_with(HashMap::new).get(id).cloned() }
}

pub fn get_font(id: &str) -> Font {
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
    unsafe { DECORATION.get_or_insert_with(HashMap::new).get(id) }
}

pub fn get_decoration(id: &str) -> &DecorationMetadata {
    try_get_decoration(id).unwrap()
}

pub fn iter_decoration() -> HashMapIter<'static, String, DecorationMetadata> {
    unsafe { DECORATION.get_or_insert_with(HashMap::new).iter() }
}

#[cfg(feature = "macroquad-backend")]
static mut IMAGES: Option<HashMap<String, ImageResource>> = None;

#[cfg(feature = "macroquad-backend")]
pub fn try_get_image(id: &str) -> Option<&ImageResource> {
    unsafe { IMAGES.get_or_insert_with(HashMap::new).get(id) }
}

#[cfg(feature = "macroquad-backend")]
pub fn get_image(id: &str) -> &ImageResource {
    try_get_image(id).unwrap()
}

#[cfg(feature = "macroquad-backend")]
pub fn iter_images() -> HashMapIter<'static, String, ImageResource> {
    unsafe { IMAGES.get_or_insert_with(HashMap::new).iter() }
}

pub struct ModLoadingIterator {
    extension: &'static str,
    active_mods: Vec<String>,
    next_i: usize,
}

impl ModLoadingIterator {
    pub async fn new<P: AsRef<Path>, E: Into<Option<&'static str>>>(
        path: P,
        ext: E,
    ) -> Result<Self> {
        let path = path.as_ref();

        set_mods_dir(path);

        let ext = ext.into().unwrap_or(DEFAULT_RESOURCE_FILE_EXTENSION);

        let active_mods_file_path = path.join(ACTIVE_MODS_FILE_NAME).with_extension(ext);

        let bytes = read_from_file(active_mods_file_path).await?;

        let active_mods: Vec<String> = deserialize_bytes_by_extension(ext, &bytes)?;

        Ok(ModLoadingIterator {
            extension: ext,
            active_mods,
            next_i: 0,
        })
    }

    pub async fn next(&mut self) -> Result<Option<(String, ModMetadata)>> {
        if self.next_i >= self.active_mods.len() {
            return Ok(None);
        }

        let current_mod = &self.active_mods[self.next_i];

        let mod_path = Path::new(&mods_dir()).join(current_mod);

        let mod_file_path = mod_path.join(MOD_FILE_NAME).with_extension(self.extension);

        let bytes = read_from_file(mod_file_path).await?;

        let meta: ModMetadata = deserialize_bytes_by_extension(self.extension, &bytes)?;

        let mut has_game_version_mismatch = false;

        if let Some(req_version) = &meta.game_version {
            if *req_version != env!("CARGO_PKG_VERSION") {
                has_game_version_mismatch = true;

                #[cfg(debug_assertions)]
                println!(
                    "WARNING: Loading mod {} (v{}) failed: Game version requirement mismatch (v{})",
                    &meta.id, &meta.version, req_version
                );
            }
        }

        if !has_game_version_mismatch {
            let mut has_unmet_dependencies = false;

            for dependency in &meta.dependencies {
                let res = loaded_mods()
                    .iter()
                    .find(|&meta| meta.id == dependency.id && meta.version == dependency.version);

                if res.is_none() {
                    has_unmet_dependencies = true;

                    #[cfg(debug_assertions)]
                    println!(
                        "WARNING: Loading mod {} (v{}) failed: Unmet dependency {} (v{})",
                        &meta.id, &meta.version, &dependency.id, &dependency.version
                    );

                    break;
                }
            }

            if !has_unmet_dependencies {
                loaded_mods_mut().push(meta.clone());

                self.next_i += 1;

                return Ok(Some((mod_path.to_string_lossy().to_string(), meta)));
            }
        }

        Ok(None)
    }
}

pub async fn load_particle_effects<P: AsRef<Path>>(
    path: P,
    ext: &str,
    is_required: bool,
    should_overwrite: bool,
) -> Result<()> {
    let particle_effects = unsafe { PARTICLE_EFFECTS.get_or_insert_with(HashMap::new) };

    if should_overwrite {
        particle_effects.clear();
    }

    let particle_effects_file_path = path
        .as_ref()
        .join(PARTICLE_EFFECTS_FILE)
        .with_extension(ext);

    match read_from_file(&particle_effects_file_path).await {
        Err(err) => {
            if is_required {
                return Err(err.into());
            }
        }
        Ok(bytes) => {
            let metadata: Vec<ParticleEffectMetadata> =
                deserialize_bytes_by_extension(ext, &bytes)?;

            for meta in metadata {
                let file_path = path.as_ref().join(&meta.path);

                let extension = file_path.extension().unwrap().to_str().unwrap();

                let bytes = read_from_file(&file_path).await?;

                let cfg: EmitterConfig = deserialize_bytes_by_extension(extension, &bytes)?;

                particle_effects.insert(meta.id, cfg);
            }
        }
    }

    Ok(())
}

pub async fn load_audio<P: AsRef<Path>>(
    path: P,
    ext: &str,
    is_required: bool,
    should_overwrite: bool,
) -> Result<()> {
    let sounds = unsafe { AUDIO.get_or_insert_with(HashMap::new) };

    if should_overwrite {
        sounds.clear();
    }

    let audio_file_path = path.as_ref().join(AUDIO_FILE).with_extension(ext);

    match read_from_file(&audio_file_path).await {
        Err(err) => {
            if is_required {
                return Err(err.into());
            }
        }
        Ok(bytes) => {
            let metadata: Vec<SoundMetadata> = deserialize_bytes_by_extension(ext, &bytes)?;

            for meta in metadata {
                let file_path = path.as_ref().join(meta.path);

                let kind = meta.kind.map(AudioKind::from).unwrap_or_default();
                let mut sound = load_sound_file(&file_path, kind).await?;

                if let Some(volume) = meta.volume_modifier {
                    sound.set_volume_modifier(volume);
                }

                sounds.insert(meta.id, sound);
            }
        }
    }

    Ok(())
}

pub async fn load_textures<P: AsRef<Path>>(
    path: P,
    ext: &str,
    is_required: bool,
    should_overwrite: bool,
) -> Result<()> {
    let textures = unsafe { TEXTURES.get_or_insert_with(HashMap::new) };

    if should_overwrite {
        textures.clear();
    }

    let textures_file_path = path.as_ref().join(TEXTURES_FILE).with_extension(ext);

    match read_from_file(&textures_file_path).await {
        Err(err) => {
            if is_required {
                return Err(err.into());
            }
        }
        Ok(bytes) => {
            let metadata: Vec<TextureMetadata> = deserialize_bytes_by_extension(ext, &bytes)?;

            for meta in metadata {
                let file_path = path.as_ref().join(&meta.path);

                let texture = load_texture_file(&file_path, meta.filter_mode).await?;

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

                let res = TextureResource { texture, meta };

                textures.insert(key, res);
            }
        }
    }

    Ok(())
}

pub async fn load_maps<P: AsRef<Path>>(
    path: P,
    ext: &str,
    is_required: bool,
    should_overwrite: bool,
) -> Result<()> {
    let maps: &mut Vec<MapResource> = unsafe { MAPS.borrow_mut() };

    if should_overwrite {
        maps.clear();
    }

    let maps_file_path = path.as_ref().join(MAPS_FILE).with_extension(ext);

    match read_from_file(&maps_file_path).await {
        Err(err) => {
            if is_required {
                return Err(err.into());
            }
        }
        Ok(bytes) => {
            let metadata: Vec<MapMetadata> = deserialize_bytes_by_extension(ext, &bytes)?;

            for meta in metadata {
                let map_path = path.as_ref().join(&meta.path);
                let preview_path = path.as_ref().join(&meta.preview_path);

                let map = if meta.is_tiled_map {
                    Map::load_tiled(map_path, None).await?
                } else {
                    Map::load(map_path).await?
                };

                let preview = load_texture_file(&preview_path, TextureFilterMode::Nearest).await?;

                let res = MapResource { map, preview, meta };

                maps.push(res)
            }
        }
    }

    Ok(())
}

pub async fn load_decoration<P: AsRef<Path>>(
    path: P,
    ext: &str,
    is_required: bool,
    should_overwrite: bool,
) -> Result<()> {
    let decoration = unsafe { DECORATION.get_or_insert_with(HashMap::new) };

    if should_overwrite {
        decoration.clear();
    }

    let decoration_file_path = path.as_ref().join(DECORATION_FILE).with_extension(ext);

    match read_from_file(&decoration_file_path).await {
        Err(err) => {
            if is_required {
                return Err(err.into());
            }
        }
        Ok(bytes) => {
            let decoration_paths: Vec<String> = deserialize_bytes_by_extension(ext, &bytes)?;

            for decoration_path in decoration_paths {
                let path = path.as_ref().join(&decoration_path);

                let extension = path.extension().unwrap().to_str().unwrap();

                let bytes = read_from_file(&path).await?;

                let params: DecorationMetadata = deserialize_bytes_by_extension(extension, &bytes)?;

                decoration.insert(params.id.clone(), params);
            }
        }
    }

    Ok(())
}

#[cfg(not(feature = "macroquad-backend"))]
pub async fn load_images<P: AsRef<Path>>(
    path: P,
    ext: &str,
    is_required: bool,
    should_overwrite: bool,
) -> Result<()> {
    Ok(())
}

#[cfg(feature = "macroquad-backend")]
pub async fn load_images<P: AsRef<Path>>(
    path: P,
    ext: &str,
    is_required: bool,
    should_overwrite: bool,
) -> Result<()> {
    let images = unsafe { IMAGES.get_or_insert_with(HashMap::new) };

    if should_overwrite {
        images.clear();
    }

    let images_file_path = path.as_ref().join(IMAGES_FILE).with_extension(ext);

    match read_from_file(&images_file_path).await {
        Err(err) => {
            if is_required {
                return Err(err.into());
            }
        }
        Ok(bytes) => {
            let metadata: Vec<ImageMetadata> = deserialize_bytes_by_extension(ext, &bytes)?;

            for meta in metadata {
                let file_path = path.as_ref().join(&meta.path);

                let image =
                    crate::macroquad::texture::load_image(&file_path.to_string_lossy()).await?;

                let key = meta.id.clone();

                let meta = ImageMetadata {
                    size: vec2(image.width() as f32, image.height() as f32),
                    ..meta
                };

                let res = ImageResource { image, meta };

                images.insert(key, res);
            }
        }
    }

    rebuild_gui_theme();

    Ok(())
}

pub async fn load_fonts<P: AsRef<Path>>(
    path: P,
    ext: &str,
    is_required: bool,
    should_overwrite: bool,
) -> Result<()> {
    let fonts = unsafe { FONTS.get_or_insert_with(HashMap::new) };

    if should_overwrite {
        fonts.clear();
    }

    let fonts_file_path = path.as_ref().join(FONTS_FILE).with_extension(ext);

    match read_from_file(&fonts_file_path).await {
        Err(err) => {
            if is_required {
                return Err(err.into());
            }
        }
        Ok(bytes) => {
            let metadata: Vec<FontMetadata> = deserialize_bytes_by_extension(ext, &bytes)?;

            for meta in metadata {
                let file_path = path.as_ref().join(&meta.path);

                let font = load_font(&file_path).await?;

                let key = meta.id.clone();

                fonts.insert(key, font);
            }
        }
    }

    rebuild_gui_theme();

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct ParticleEffectMetadata {
    id: String,
    path: String,
}

#[derive(Serialize, Deserialize)]
struct SoundMetadata {
    id: String,
    /// This determines what volume that will be applied to this sound resource.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
    /// This can be used to modify the volume of this individual sound resource.
    /// It should be a value between 0.0 and 1.0.
    /// Master volume and the volume setting for this particular sound type will still be applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    volume_modifier: Option<f32>,
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
pub struct FontMetadata {
    pub id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMetadata {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub path: String,
    pub preview_path: String,
    #[serde(default, skip_serializing_if = "crate::parsing::is_false")]
    pub is_tiled_map: bool,
    #[serde(default, skip_serializing_if = "crate::parsing::is_false")]
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
    let assets_dir = assets_dir();
    let export_dir = Path::new(&assets_dir).join(&map_resource.meta.path);

    {
        let maps: &mut Vec<MapResource> = unsafe { MAPS.borrow_mut() };

        if export_dir.exists() {
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

        map_resource.map.save(export_dir)?;

        maps.push(map_resource.clone());
    }

    save_maps_file()?;

    Ok(())
}

pub fn delete_map(index: usize) -> Result<()> {
    let map_resource = unsafe { MAPS.remove(index) };

    let assets_dir = assets_dir();
    let path = Path::new(&assets_dir).join(&map_resource.meta.path);

    fs::remove_file(path)?;

    save_maps_file()?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn save_maps_file() -> Result<()> {
    let assets_dir = assets_dir();
    let maps_file_path = Path::new(&assets_dir)
        .join(MAPS_FILE)
        .with_extension(DEFAULT_RESOURCE_FILE_EXTENSION);

    let metadata: Vec<MapMetadata> = iter_maps().map(|res| res.meta.clone()).collect();

    let str = serde_json::to_string_pretty(&metadata)?;
    fs::write(maps_file_path, &str)?;

    Ok(())
}

pub fn is_valid_map_export_path<P: AsRef<Path>>(path: P, should_overwrite: bool) -> bool {
    let path = path.as_ref();

    if let Some(file_name) = path.file_name() {
        if is_valid_map_file_name(&file_name.to_string_lossy().to_string()) {
            let res = iter_maps().find(|res| Path::new(&res.meta.path) == path);

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
