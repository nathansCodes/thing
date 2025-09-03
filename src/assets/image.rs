use iced::widget::image;

use crate::assets::Asset;

const DEFAULT_IMAGE: &[u8] = include_bytes!("../../assets/default.png").as_slice();

#[derive(Clone, Debug)]
pub struct Image {
    pub handle: image::Handle,
}

impl Image {
    pub fn new(handle: image::Handle) -> Self {
        Self { handle }
    }
}

impl<'a> TryFrom<&'a Asset> for &'a Image {
    type Error = ();

    #[allow(unreachable_patterns)]
    fn try_from(asset: &'a Asset) -> Result<Self, Self::Error> {
        match asset {
            Asset::Image(image) => Ok(image),
            _ => Err(()),
        }
    }
}

impl From<Image> for Asset {
    fn from(image: Image) -> Self {
        Asset::Image(image)
    }
}

pub fn default_image() -> image::Handle {
    image::Handle::from_bytes(DEFAULT_IMAGE)
}
