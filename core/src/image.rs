use std::collections::hash_map::{IntoIter, Iter};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use serde::{Deserialize, Serialize};

pub use crate::backend_impl::image::*;

use crate::file::read_from_file;
use crate::gui::rebuild_gui_theme;
use crate::parsing::deserialize_bytes_by_extension;

use crate::math::vec2;
use crate::result::Result;

pub const IMAGE_RESOURCES_FILE: &str = "images";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Pnm,
    Tiff,
    Tga,
    Dds,
    Bmp,
    Ico,
    Hdr,
    Farbfeld,
    Avif,
}

#[derive(Clone)]
pub struct Image(ImageImpl);

impl Image {
    pub fn from_bytes<T>(bytes: &[u8], format: T) -> Result<Self>
    where
        T: Into<Option<ImageFormat>>,
    {
        let image_impl = ImageImpl::from_bytes(bytes, format)?;
        Ok(Image(image_impl))
    }

    pub async fn from_file<P, T>(path: P, format: T) -> Result<Self>
    where
        P: AsRef<Path>,
        T: Into<Option<ImageFormat>>,
    {
        let bytes = read_from_file(path).await?;
        Self::from_bytes(&bytes, format)
    }

    pub(crate) fn set_id(&self, id: &str) {
        image_map().insert(id.to_string(), self.clone());
    }
}

static mut IMAGES: Option<HashMap<String, Image>> = None;

fn image_map() -> &'static mut HashMap<String, Image> {
    unsafe { IMAGES.get_or_insert_with(HashMap::new) }
}

impl Deref for Image {
    type Target = ImageImpl;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Image {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn try_get_image(id: &str) -> Option<Image> {
    image_map().get(id).cloned()
}

pub fn get_image(id: &str) -> Image {
    try_get_image(id).unwrap()
}

pub fn iter_images() -> Iter<'static, String, Image> {
    image_map().iter()
}

pub fn load_image_bytes<T>(bytes: &[u8], format: T) -> Result<Image>
where
    T: Into<Option<ImageFormat>>,
{
    Image::from_bytes(bytes, format)
}

pub async fn load_image_file<P, T>(path: P, format: T) -> Result<Image>
where
    P: AsRef<Path>,
    T: Into<Option<ImageFormat>>,
{
    Image::from_file(path, format).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub id: String,
    pub path: String,
    #[serde(
        default,
        rename = "format_override",
        skip_serializing_if = "Option::is_none"
    )]
    pub format: Option<ImageFormat>,
}

pub async fn load_images<P: AsRef<Path>>(
    path: P,
    ext: &str,
    is_required: bool,
    should_overwrite: bool,
) -> Result<()> {
    let images = image_map();

    if should_overwrite {
        images.clear();
    }

    let images_file_path = path.as_ref().join(IMAGE_RESOURCES_FILE).with_extension(ext);

    match read_from_file(&images_file_path).await {
        Err(err) => {
            if is_required {
                return Err(err.into());
            }
        }
        Ok(bytes) => {
            let metadata: Vec<ImageMetadata> = deserialize_bytes_by_extension(ext, &bytes)?;

            for meta in metadata {
                let path = path.as_ref().join(&meta.path);

                let image = Image::from_file(path, meta.format).await?;

                image.set_id(&meta.id);
            }
        }
    }

    rebuild_gui_theme();

    Ok(())
}
