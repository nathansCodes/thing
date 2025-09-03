use serde::{Deserialize, Serialize};

use crate::assets::{Asset, AssetHandle};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub img: AssetHandle,
}

impl<'a> TryFrom<&'a Asset> for &'a Character {
    type Error = ();

    #[allow(unreachable_patterns)]
    fn try_from(asset: &'a Asset) -> Result<Self, Self::Error> {
        match asset {
            Asset::Character(chara) => Ok(chara),
            _ => Err(()),
        }
    }
}

impl From<Character> for Asset {
    fn from(value: Character) -> Self {
        Asset::Character(value)
    }
}
