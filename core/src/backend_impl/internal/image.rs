use std::collections::HashMap;
use std::ffi::OsStr;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use crate::image::{ImageFormat, ImagePixelFormat};
use image::DynamicImage;

use crate::result::Result;

impl From<ImageFormat> for image::ImageFormat {
    fn from(format: ImageFormat) -> Self {
        match format {
            ImageFormat::Png => Self::Png,
            ImageFormat::Jpeg => Self::Jpeg,
            ImageFormat::Gif => Self::Gif,
            ImageFormat::WebP => Self::WebP,
            ImageFormat::Pnm => Self::Pnm,
            ImageFormat::Tiff => Self::Tiff,
            ImageFormat::Tga => Self::Tga,
            ImageFormat::Dds => Self::Dds,
            ImageFormat::Bmp => Self::Bmp,
            ImageFormat::Ico => Self::Ico,
            ImageFormat::Hdr => Self::Hdr,
            ImageFormat::Farbfeld => Self::Farbfeld,
            ImageFormat::Avif => Self::Avif,
        }
    }
}

impl ImageFormat {
    fn from_crate(format: image::ImageFormat) -> Option<Self> {
        match format {
            image::ImageFormat::Png => Some(Self::Png),
            image::ImageFormat::Jpeg => Some(Self::Jpeg),
            image::ImageFormat::Gif => Some(Self::Gif),
            image::ImageFormat::WebP => Some(Self::WebP),
            image::ImageFormat::Pnm => Some(Self::Pnm),
            image::ImageFormat::Tiff => Some(Self::Tiff),
            image::ImageFormat::Tga => Some(Self::Tga),
            image::ImageFormat::Dds => Some(Self::Dds),
            image::ImageFormat::Bmp => Some(Self::Bmp),
            image::ImageFormat::Ico => Some(Self::Ico),
            image::ImageFormat::Hdr => Some(Self::Hdr),
            image::ImageFormat::Farbfeld => Some(Self::Farbfeld),
            image::ImageFormat::Avif => Some(Self::Avif),
            _ => None,
        }
    }

    pub fn from_extension<S>(ext: S) -> Option<Self>
    where
        S: AsRef<OsStr>,
    {
        image::ImageFormat::from_extension(ext)
            .map(|f| Self::from_crate(f))
            .flatten()
    }
}

pub struct ImageImpl(DynamicImage);

impl ImageImpl {
    pub(crate) fn from_bytes<T>(bytes: &[u8], format: T) -> Result<Self>
    where
        T: Into<Option<ImageFormat>>,
    {
        let mut dyn_image = if let Some(format) = format.into() {
            image::load_from_memory_with_format(bytes, format.into())?
        } else {
            image::load_from_memory(bytes)?
        };

        Ok(ImageImpl(dyn_image))
    }

    pub fn as_dyn(&self) -> &DynamicImage {
        &self.0
    }

    pub fn as_dyn_mut(&mut self) -> &mut DynamicImage {
        &mut self.0
    }
}
