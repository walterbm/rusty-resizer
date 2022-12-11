use image::ImageFormat;
use serde::Deserialize;

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResizeImageFormat {
    Auto,
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
    OpenExr,
    Farbfeld,
    Avif,
}

impl From<ResizeImageFormat> for Option<ImageFormat> {
    fn from(resize_image_format: ResizeImageFormat) -> Option<ImageFormat> {
        match resize_image_format {
            ResizeImageFormat::Png => Some(ImageFormat::Png),
            ResizeImageFormat::Jpeg => Some(ImageFormat::Jpeg),
            ResizeImageFormat::Gif => Some(ImageFormat::Gif),
            ResizeImageFormat::WebP => Some(ImageFormat::WebP),
            ResizeImageFormat::Pnm => Some(ImageFormat::Pnm),
            ResizeImageFormat::Tiff => Some(ImageFormat::Tiff),
            ResizeImageFormat::Tga => Some(ImageFormat::Tga),
            ResizeImageFormat::Dds => Some(ImageFormat::Dds),
            ResizeImageFormat::Bmp => Some(ImageFormat::Bmp),
            ResizeImageFormat::Ico => Some(ImageFormat::Ico),
            ResizeImageFormat::Hdr => Some(ImageFormat::Hdr),
            ResizeImageFormat::OpenExr => Some(ImageFormat::OpenExr),
            ResizeImageFormat::Farbfeld => Some(ImageFormat::Farbfeld),
            ResizeImageFormat::Avif => Some(ImageFormat::Avif),
            // can not convert ResizeImageFormat::Auto into an ImageFormat
            ResizeImageFormat::Auto => None,
        }
    }
}
