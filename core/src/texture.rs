use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::slice::Iter;
use std::vec::IntoIter;

use serde::{Deserialize, Serialize};

use crate::math::{Size, Vec2};
use crate::result::Result;

pub use crate::backend_impl::texture::*;

use crate::file::read_from_file;
use crate::parsing::deserialize_bytes_by_extension;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureKind {
    Unknown,
    Background,
    Tileset,
    Spritesheet,
}

impl Default for TextureKind {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorFormat {
    Rgb,
    Rgba,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureFilterMode {
    Linear,
    Nearest,
}

impl Default for TextureFilterMode {
    fn default() -> Self {
        Self::Nearest
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Texture2D(usize);

impl Texture2D {
    pub fn from_bytes<T, K, F, S>(
        bytes: &[u8],
        format: T,
        kind: K,
        filter_mode: F,
        frame_size: S,
    ) -> Result<Texture2D>
    where
        T: Into<Option<TextureFormat>>,
        K: Into<Option<TextureKind>>,
        F: Into<Option<TextureFilterMode>>,
        S: Into<Option<Size<f32>>>,
    {
        let texture_impl = Texture2DImpl::from_bytes(bytes, format, kind, filter_mode, frame_size)?;
        Ok(add_texture_to_map(texture_impl))
    }

    pub async fn from_file<P, T, K, F, S>(
        path: P,
        format: T,
        kind: K,
        filter_mode: F,
        frame_size: S,
    ) -> Result<Texture2D>
    where
        P: AsRef<Path>,
        T: Into<Option<TextureFormat>>,
        K: Into<Option<TextureKind>>,
        F: Into<Option<TextureFilterMode>>,
        S: Into<Option<Size<f32>>>,
    {
        let bytes = read_from_file(path).await?;
        Self::from_bytes(&bytes, format, kind, filter_mode, frame_size)
    }

    pub(crate) fn set_id(&self, id: &str) {
        unsafe { TEXTURE_IDS.get_or_insert_with(HashMap::new) }.insert(id.to_string(), self.0);
    }
}

impl Deref for Texture2D {
    type Target = Texture2DImpl;

    fn deref(&self) -> &Self::Target {
        texture_map().get(&self.0).unwrap()
    }
}

impl DerefMut for Texture2D {
    fn deref_mut(&mut self) -> &mut Self::Target {
        texture_map().get_mut(&self.0).unwrap()
    }
}

static mut NEXT_TEXTURE_INDEX: usize = 0;
static mut TEXTURES: Option<HashMap<usize, Texture2DImpl>> = None;

fn texture_map() -> &'static mut HashMap<usize, Texture2DImpl> {
    unsafe { TEXTURES.get_or_insert_with(HashMap::new) }
}

fn add_texture_to_map(texture_impl: Texture2DImpl) -> Texture2D {
    let index = unsafe { NEXT_TEXTURE_INDEX };
    texture_map().insert(index, texture_impl);
    unsafe { NEXT_TEXTURE_INDEX += 1 };
    Texture2D(index)
}

pub fn iter_textures() -> IntoIter<Texture2D> {
    texture_map()
        .keys()
        .map(|index| Texture2D(*index))
        .collect::<Vec<_>>()
        .into_iter()
}

pub fn iter_textures_of_kind(kind: TextureKind) -> IntoIter<Texture2D> {
    texture_map()
        .keys()
        .filter_map(|&index| {
            let texture = Texture2D(index);
            if texture.kind == kind {
                Some(texture)
            } else {
                None
            }
        })
        .collect::<Vec<Texture2D>>()
        .into_iter()
}

static mut TEXTURE_IDS: Option<HashMap<String, usize>> = None;

pub fn try_get_texture(id: &str) -> Option<Texture2D> {
    if let Some(index) = unsafe { TEXTURE_IDS.get_or_insert_with(HashMap::new) }.get(id) {
        Some(Texture2D(*index))
    } else {
        None
    }
}

pub fn get_texture(id: &str) -> Texture2D {
    try_get_texture(id).unwrap()
}

pub fn iter_textures_with_ids() -> std::collections::hash_map::IntoIter<String, Texture2D> {
    unsafe { TEXTURE_IDS.get_or_insert_with(HashMap::new) }
        .iter()
        .map(|(k, v)| (k.to_string(), Texture2D(*v)))
        .collect::<HashMap<_, _>>()
        .into_iter()
}

pub fn iter_texture_ids() -> IntoIter<String> {
    unsafe { TEXTURE_IDS.get_or_insert_with(HashMap::new) }
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
}

pub fn iter_texture_ids_of_kind(kind: TextureKind) -> IntoIter<String> {
    iter_textures_with_ids()
        .filter_map(|(id, texture)| {
            if texture.kind == kind {
                Some(id.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
}

pub fn load_texture_bytes<T, K, F, S>(
    bytes: &[u8],
    format: T,
    kind: K,
    filter_mode: F,
    frame_size: S,
) -> Result<Texture2D>
where
    T: Into<Option<TextureFormat>>,
    K: Into<Option<TextureKind>>,
    F: Into<Option<TextureFilterMode>>,
    S: Into<Option<Size<f32>>>,
{
    Texture2D::from_bytes(bytes, format, kind, filter_mode, frame_size)
}

pub async fn load_texture_file<P, T, K, F, S>(
    path: P,
    format: T,
    kind: K,
    filter_mode: F,
    frame_size: S,
) -> Result<Texture2D>
where
    P: AsRef<Path>,
    T: Into<Option<TextureFormat>>,
    K: Into<Option<TextureKind>>,
    F: Into<Option<TextureFilterMode>>,
    S: Into<Option<Size<f32>>>,
{
    Texture2D::from_file(path, format, kind, filter_mode, frame_size).await
}

const TEXTURE_RESOURCES_FILE: &str = "textures";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureMetadata {
    pub id: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<TextureFormat>,
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

    let textures_file_path = path
        .as_ref()
        .join(TEXTURE_RESOURCES_FILE)
        .with_extension(ext);

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

                let texture = load_texture_file(
                    &file_path,
                    meta.format,
                    meta.kind,
                    meta.filter_mode,
                    meta.frame_size,
                )
                .await?;

                texture.set_id(&meta.id);
            }
        }
    }

    Ok(())
}
