use std::io::{Cursor, Write};

use iced::{
    advanced::graphics::image::image_rs::{ImageFormat, RgbaImage},
    widget::image as iced_image,
};

use crate::assets::{AsBytes, Asset};

const DEFAULT_IMAGE: &[u8] = include_bytes!("../../assets/default.png").as_slice();

#[derive(Clone, Debug)]
pub struct Image {
    format: ImageFormat,
    pub handle: iced_image::Handle,
}

impl Image {
    pub fn new(format: ImageFormat, handle: iced_image::Handle) -> Self {
        Self { format, handle }
    }
}

impl AsBytes for Image {
    fn as_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = Cursor::new(Vec::new());

        match &self.handle {
            iced_image::Handle::Path(..) => (),
            iced_image::Handle::Bytes(_, img_bytes) => {
                bytes.write_all(img_bytes)?;
            }
            iced_image::Handle::Rgba {
                id: _,
                width,
                height,
                pixels,
            } => {
                if let Some(img) = RgbaImage::from_raw(*width, *height, pixels.to_vec()) {
                    img.write_to(&mut bytes, self.format)?;
                }
            }
        };

        Ok(bytes.into_inner())
    }
}

impl<'a> TryFrom<&'a Asset> for &'a Image {
    type Error = ();

    fn try_from(asset: &'a Asset) -> Result<Self, Self::Error> {
        if let Asset::Image(image) = asset {
            Ok(image)
        } else {
            Err(())
        }
    }
}

impl From<Image> for Asset {
    fn from(image: Image) -> Self {
        Asset::Image(image)
    }
}

pub fn default_image() -> Image {
    Image::new(
        ImageFormat::Png,
        iced_image::Handle::from_bytes(DEFAULT_IMAGE),
    )
}
