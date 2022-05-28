use image::ImageFormat;
use serde::{Deserialize, Deserializer};

/// An mirror implementation of the image::ImageFormat crate enum
/// for use when deserializing query parameters
/// see: https://docs.rs/image/latest/image/enum.ImageFormat.html
#[derive(Deserialize)]
#[serde(rename_all = "lowercase", remote = "ImageFormat")]
enum ImageFormatDef {
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

/// Helper to deserialize a query parameter into an ImageFormat
/// since Serde does not support a way to use "deserialize_with"
/// when the data is inside an Option
/// see: https://github.com/serde-rs/serde/issues/723
pub fn deserialize_image_format_external_enum<'de, D>(
    deserializer: D,
) -> Result<Option<ImageFormat>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Helper(#[serde(with = "ImageFormatDef")] ImageFormat);

    let helper = Option::deserialize(deserializer)?;
    Ok(helper.map(|Helper(external)| external))
}
