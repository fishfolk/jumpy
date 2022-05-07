use std::collections::hash_map::IntoIter;
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

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImagePixelFormat {
    Luma8,
    Lumaa8,
    Rgb8,
    Rgba8,
    Luma16,
    Lumaa16,
    Rgb16,
    Rgba16,
    Rgb32f,
    Rgba32f,
}

pub struct Image(usize);

impl Image {
    pub fn from_bytes<T>(bytes: &[u8], format: T) -> Result<Self>
    where
        T: Into<Option<ImageFormat>>,
    {
        let image_impl = ImageImpl::from_bytes(bytes, format)?;
        Ok(add_image_to_map(image_impl))
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
        unsafe { IMAGE_IDS.get_or_insert_with(HashMap::new) }.insert(id.to_string(), self.0);
    }
}

static mut NEXT_IMAGE_INDEX: usize = 0;
static mut IMAGES: Option<HashMap<usize, ImageImpl>> = None;

fn image_map() -> &'static mut HashMap<usize, ImageImpl> {
    unsafe { IMAGES.get_or_insert_with(HashMap::new) }
}

fn add_image_to_map(image_impl: ImageImpl) -> Image {
    let index = unsafe { NEXT_IMAGE_INDEX };
    image_map().insert(index, image_impl);
    unsafe { NEXT_IMAGE_INDEX += 1 };
    Image(index)
}

impl Deref for Image {
    type Target = ImageImpl;

    fn deref(&self) -> &Self::Target {
        image_map().get(&self.0).unwrap()
    }
}

impl DerefMut for Image {
    fn deref_mut(&mut self) -> &mut Self::Target {
        image_map().get_mut(&self.0).unwrap()
    }
}

static mut IMAGE_IDS: Option<HashMap<String, usize>> = None;

pub fn try_get_image(id: &str) -> Option<Image> {
    if let Some(index) = unsafe { IMAGE_IDS.get_or_insert_with(HashMap::new) }.get(id) {
        Some(Image(*index))
    } else {
        None
    }
}

pub fn get_image(id: &str) -> Image {
    try_get_image(id).unwrap()
}

pub fn iter_images() -> IntoIter<String, Image> {
    unsafe {
        IMAGE_IDS
            .get_or_insert_with(HashMap::new)
            .iter()
            .map(|(k, v)| (k.to_string(), Image(*v)))
            .collect::<HashMap<_, _>>()
            .into_iter()
    }
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
    let images = unsafe { IMAGES.get_or_insert_with(HashMap::new) };

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
