use iced::widget::image;
use serde::{Deserialize, Serialize};

use crate::assets::Asset;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Image {
    pub file_name: String,
    #[serde(skip)]
    #[serde(default = "default_image")]
    pub handle: image::Handle,
}

impl TryFrom<&Asset> for Image {
    type Error = ();

    #[allow(unreachable_patterns)]
    fn try_from(asset: &Asset) -> Result<Self, Self::Error> {
        match asset {
            Asset::Image(image) => Ok(image.clone()),
            _ => Err(()),
        }
    }
}

impl From<Image> for Asset {
    fn from(image: Image) -> Self {
        Asset::Image(image)
    }
}

fn default_image() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../assets/default.png").as_slice())
}
