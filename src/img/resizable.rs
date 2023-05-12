use actix_web::web::Bytes;
use image::ImageFormat;
use magick_rust::MagickWand;
use std::cmp;

use super::ImageError;

pub struct ResizableImage {
    wand: MagickWand,
}

impl ResizableImage {
    pub fn from_bytes(bytes: &Bytes) -> Result<Self, ImageError> {
        let wand = MagickWand::new();
        match wand.read_image_blob(bytes) {
            Ok(_) => Ok(Self { wand }),
            Err(_) => Err(ImageError::InvalidImage),
        }
    }

    pub fn resize(&self, width: Option<usize>, height: Option<usize>) {
        let (width, height) = match (width, height) {
            (Some(width), Some(height)) => (
                cmp::min(width, self.scale_width(height)),
                cmp::min(height, self.scale_height(width)),
            ),
            (Some(width), None) => (width, self.scale_height(width)),
            (None, Some(height)) => (self.scale_width(height), height),
            (None, None) => (self.wand.get_image_width(), self.wand.get_image_height()),
        };

        if !(width == self.wand.get_image_width() && height == self.wand.get_image_height()) {
            // if image has multiple frames use fit to resize the entire scene
            if self.wand.get_image_scene() > 0 {
                self.wand.fit(width, height);
            } else {
                self.wand.thumbnail_image(width, height);
            }
        }
    }

    fn scale_width(&self, height: usize) -> usize {
        (self.wand.get_image_width() as f64 * (height as f64 / self.wand.get_image_height() as f64))
            as usize
    }

    fn scale_height(&self, width: usize) -> usize {
        (self.wand.get_image_height() as f64 * (width as f64 / self.wand.get_image_width() as f64))
            as usize
    }

    pub fn to_buffer_mut(
        &mut self,
        quality: u8,
        format: ImageFormat,
    ) -> Result<Vec<u8>, ImageError> {
        self.wand
            .strip_image()
            .map_err(|_| ImageError::FailedWrite)?;

        self.wand
            .set_image_compression_quality(quality as usize)
            .map_err(|_| ImageError::FailedWrite)?;

        self.wand
            .write_images_blob(format.extensions_str()[0])
            .map_err(|_| ImageError::FailedWrite)
    }

    pub fn format(&self) -> Result<ImageFormat, ImageError> {
        self.wand
            .get_image_format()
            .map_err(|_| ImageError::InvalidFormat)
            .and_then(|format| ImageFormat::from_extension(format).ok_or(ImageError::InvalidFormat))
    }

    pub fn mime_type(self) -> Result<&'static str, ImageError> {
        let format = self.format()?;

        match format {
            ImageFormat::Avif => Ok("image/avif"),
            ImageFormat::Jpeg => Ok("image/jpeg"),
            ImageFormat::Png => Ok("image/png"),
            ImageFormat::Gif => Ok("image/gif"),
            ImageFormat::WebP => Ok("image/webp"),
            ImageFormat::Tiff => Ok("image/tiff"),
            ImageFormat::Tga => Ok("image/x-tga"),
            ImageFormat::Dds => Ok("image/vnd-ms.dds"),
            ImageFormat::Bmp => Ok("image/bmp"),
            ImageFormat::Ico => Ok("image/x-icon"),
            ImageFormat::Hdr => Ok("image/vnd.radiance"),
            ImageFormat::OpenExr => Ok("image/x-exr"),
            ImageFormat::Pnm => Ok("image/x-portable-bitmap"),
            _ => Ok("application/octet-stream"),
        }
    }
}
