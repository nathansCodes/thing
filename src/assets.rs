mod asset_path;
mod image;
mod ui;

pub use asset_path::AssetPath;
pub use image::Image;
pub use ui::{update, view};

use std::{collections::HashMap, ops::Index, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::io::IOError;

#[derive(Debug, Clone)]
pub enum Asset {
    Image(Image),
}

impl Asset {
    pub fn folder(&self) -> String {
        match self {
            Self::Image(_) => "images".to_string(),
        }
    }

    #[allow(unused)]
    pub fn kind(&self) -> AssetKind {
        match self {
            Self::Image(_) => AssetKind::Image,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AssetHandle(u32);

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AssetKind {
    #[default]
    Image,
}

impl AssetKind {
    pub fn folder(&self) -> &'static str {
        match self {
            Self::Image => "images",
        }
    }

    // NOTE: update this whenever a new kind of asset is added
    // tried forcing this by also making use of a match but I can't because of temporary value
    // shenanigans
    pub fn folders() -> &'static [&'static str] {
        &["images"]
    }
}

impl TryFrom<&str> for AssetKind {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "images" | "image" => Ok(Self::Image),
            _ => Err(()),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum ViewMode {
    #[default]
    Thumbnails,
    List,
}

#[derive(Default)]
pub struct AssetsData {
    view: AssetKind,
    view_mode: ViewMode,
    view_dropdown_open: bool,
    assets: HashMap<AssetPath, Asset>,
    index: HashMap<u32, AssetPath>,
    error_state: Option<IOError>,
    filter: String,
    folder: Option<PathBuf>,
}

impl Index<AssetHandle> for AssetsData {
    type Output = Asset;

    fn index(&self, index: AssetHandle) -> &Self::Output {
        &self.assets[&self.index[&index.0]]
    }
}

impl AssetsData {
    pub fn get(&self, handle: AssetHandle) -> Option<&Asset> {
        self.index.get(&handle.0).and_then(|asset_path| {
            println!("{asset_path:?}");
            self.assets.get(asset_path)
        })
    }

    #[allow(unused)]
    pub fn handle(&self, asset_path: AssetPath) -> Option<AssetHandle> {
        self.assets.get(&asset_path).and_then(|_| {
            self.index
                .iter()
                .find_map(|(id, path)| path.eq(&asset_path).then_some(AssetHandle(*id)))
        })
    }

    pub fn add(
        &mut self,
        file_name: impl Into<String>,
        asset: impl Into<Asset>,
    ) -> Result<AssetHandle, IOError> {
        let folder = self.folder.as_ref().ok_or(IOError::NoFolderLoaded)?;

        let asset: Asset = asset.into();

        let asset_path = asset.kind() + file_name.into();

        let path = folder.join(asset_path.to_string());

        if !std::fs::exists(&path).is_ok_and(|exists| exists) {
            return Err(IOError::OSError(2));
        }

        let ids: Vec<&u32> = self.index.keys().collect();
        let id = (0..).into_iter().find(|id| !ids.contains(&id)).unwrap();

        self.assets.insert(asset_path.clone(), asset);
        self.index.insert(id, asset_path);

        Ok(AssetHandle(id))
    }

    pub fn folder(&self) -> Option<&PathBuf> {
        self.folder.as_ref()
    }

    pub fn set_folder(&mut self, folder: PathBuf) {
        self.folder = Some(folder);
    }
}

#[derive(Clone, Debug)]
pub enum AssetsMessage {
    LoadAssets(PathBuf),
    LoadCompleted(HashMap<u32, (AssetPath, Asset)>),
    LoadFailed(IOError),
    OpenAsset(AssetHandle),
    SetPayload(Option<crate::Draggable>),
    FilterChanged(String),
    ViewChanged(ViewMode),
    ShowHideDropdown,
}
