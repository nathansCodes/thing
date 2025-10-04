use serde::{Deserialize, Serialize};

use crate::{Asset, AssetHandle};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub img: AssetHandle,
}

impl<'a> TryFrom<&'a Asset> for &'a Character {
    type Error = ();

    fn try_from(asset: &'a Asset) -> Result<Self, Self::Error> {
        if let Asset::Character(chara) = asset {
            Ok(chara)
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<&'a mut Asset> for &'a mut Character {
    type Error = ();

    fn try_from(asset: &'a mut Asset) -> Result<Self, Self::Error> {
        if let Asset::Character(chara) = asset {
            Ok(chara)
        } else {
            Err(())
        }
    }
}

impl From<Character> for Asset {
    fn from(value: Character) -> Self {
        Asset::Character(value)
    }
}
