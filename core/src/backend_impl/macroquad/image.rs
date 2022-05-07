use std::ffi::OsStr;
use std::ops::{Deref, DerefMut};

pub use macroquad::texture::Image as MQImage;

use crate::image::{ImageFormat, ImagePixelFormat};
use crate::result::Result;

pub struct ImageImpl(macroquad::texture::Image);

impl ImageFormat {
    fn from_crate(format: macroquad::prelude::ImageFormat) -> Option<Self> {
        match format {
            macroquad::prelude::ImageFormat::Png => Some(Self::Png),
            macroquad::prelude::ImageFormat::Jpeg => Some(Self::Jpeg),
            macroquad::prelude::ImageFormat::Gif => Some(Self::Gif),
            macroquad::prelude::ImageFormat::WebP => Some(Self::WebP),
            macroquad::prelude::ImageFormat::Pnm => Some(Self::Pnm),
            macroquad::prelude::ImageFormat::Tiff => Some(Self::Tiff),
            macroquad::prelude::ImageFormat::Tga => Some(Self::Tga),
            macroquad::prelude::ImageFormat::Dds => Some(Self::Dds),
            macroquad::prelude::ImageFormat::Bmp => Some(Self::Bmp),
            macroquad::prelude::ImageFormat::Ico => Some(Self::Ico),
            macroquad::prelude::ImageFormat::Hdr => Some(Self::Hdr),
            macroquad::prelude::ImageFormat::Farbfeld => Some(Self::Farbfeld),
            macroquad::prelude::ImageFormat::Avif => Some(Self::Avif),
            _ => None,
        }
    }

    pub fn from_extension<S>(ext: S) -> Option<Self>
    where
        S: AsRef<OsStr>,
    {
        macroquad::prelude::ImageFormat::from_extension(ext)
            .map(|f| Self::from_crate(f))
            .flatten()
    }
}

impl From<ImageFormat> for macroquad::prelude::ImageFormat {
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

impl ImageImpl {
    pub(crate) fn from_bytes<T>(bytes: &[u8], format: T) -> Result<Self>
    where
        T: Into<Option<ImageFormat>>,
    {
        let format = format.into().map(|f| f.into());
        let mq_image = macroquad::texture::Image::from_file_with_format(bytes, format);
        Ok(ImageImpl(mq_image))
    }
}

impl Deref for ImageImpl {
    type Target = MQImage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ImageImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
