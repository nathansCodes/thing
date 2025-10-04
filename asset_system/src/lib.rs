mod asset_path;
mod character;
pub mod image;
pub mod io;

pub use asset_path::AssetPath;
pub use character::Character;
pub use image::Image;
use ron::ser::PrettyConfig;

use std::{
    collections::HashMap,
    ops::Index,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

use io::{AssetsError, load_dir};

#[derive(Debug, Clone)]
pub enum Asset {
    Image(Image),
    Character(Character),
}

impl Asset {
    pub fn folder(&self) -> &'static str {
        match self {
            Self::Image(_) => "images",
            Self::Character(_) => "characters",
        }
    }

    #[allow(unused)]
    pub fn kind(&self) -> AssetKind {
        match self {
            Self::Image(_) => AssetKind::Image,
            Self::Character(_) => AssetKind::Character,
        }
    }

    fn inner(&self) -> &dyn AsBytes {
        match self {
            Asset::Image(image) => image,
            Asset::Character(character) => character,
        }
    }
}

impl AsBytes for Asset {
    fn as_bytes(&self) -> Result<Vec<u8>> {
        self.inner().as_bytes()
    }
}

trait AsBytes {
    fn as_bytes(&self) -> Result<Vec<u8>>;
}

impl<T: Serialize> AsBytes for T {
    fn as_bytes(&self) -> Result<Vec<u8>> {
        ron::ser::to_string_pretty(self, PrettyConfig::new())
            .map(|string| string.as_bytes().to_vec())
            .map_err(|err| anyhow!(err))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetHandle(u32);

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AssetKind {
    #[default]
    Image,
    Character,
}

impl AssetKind {
    pub fn folder(&self) -> &'static str {
        match self {
            Self::Image => "images",
            Self::Character => "characters",
        }
    }

    // NOTE: update this whenever a new kind of asset is added
    // tried forcing this by also making use of a match but I can't because of temporary value
    // shenanigans
    pub fn all() -> &'static [Self] {
        &[Self::Image, Self::Character]
    }
}

impl FromStr for AssetKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "images" | "image" => Ok(Self::Image),
            "character" | "characters" => Ok(Self::Character),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for AssetKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AssetKind::Image => "image",
            AssetKind::Character => "character",
        })
    }
}

#[derive(Default)]
pub struct AssetsData {
    assets: HashMap<AssetPath, Asset>,
    index: HashMap<u32, AssetPath>,
    folder: Option<PathBuf>,
    failed_loads: Vec<(AssetPath, anyhow::Error)>,
}

impl Index<AssetHandle> for AssetsData {
    type Output = Asset;

    fn index(&self, index: AssetHandle) -> &Self::Output {
        &self.assets[&self.index[&index.0]]
    }
}

impl AssetsData {
    pub fn get(&self, handle: AssetHandle) -> Option<&Asset> {
        self.index
            .get(&handle.0)
            .and_then(|asset_path| self.assets.get(asset_path))
    }

    pub fn get_mut(&mut self, handle: AssetHandle) -> Option<&mut Asset> {
        self.index
            .get(&handle.0)
            .and_then(|asset_path| self.assets.get_mut(asset_path))
    }

    pub fn get_all_of_type<'assets: 'asset, 'asset, A>(
        &'assets self,
    ) -> Vec<(&'asset AssetPath, &'asset A)>
    where
        &'asset A: TryFrom<&'asset Asset>,
    {
        self.assets
            .iter()
            .filter_map(|(path, asset)| <&A>::try_from(asset).ok().map(|asset| (path, asset)))
            .collect()
    }

    pub fn get_direct<'assets: 'asset, 'asset, A>(
        &'assets self,
        handle: AssetHandle,
    ) -> Option<&'asset A>
    where
        &'asset A: TryFrom<&'asset Asset>,
    {
        self.index.get(&handle.0).and_then(|asset_path| {
            self.assets
                .get(asset_path)
                .and_then(|asset| <&A>::try_from(asset).ok())
        })
    }

    pub fn get_direct_mut<'assets: 'asset, 'asset, A>(
        &'assets mut self,
        handle: AssetHandle,
    ) -> Option<&'asset mut A>
    where
        &'asset mut A: TryFrom<&'asset mut Asset>,
    {
        self.index.get(&handle.0).and_then(|asset_path| {
            self.assets
                .get_mut(asset_path)
                .and_then(|asset| <&mut A>::try_from(asset).ok())
        })
    }

    pub fn is<'assets: 'asset, 'asset, A>(&'assets self, handle: AssetHandle) -> bool
    where
        &'asset A: TryFrom<&'asset Asset> + 'asset,
    {
        self.index.get(&handle.0).is_some_and(|asset_path| {
            self.assets
                .get(asset_path)
                .is_some_and(|asset| <&A>::try_from(asset).is_ok())
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

    pub fn path(&self, handle: AssetHandle) -> Option<&AssetPath> {
        self.index.get(&handle.0)
    }

    pub fn add(
        &mut self,
        file_name: impl Into<String>,
        asset: impl Into<Asset>,
    ) -> Result<AssetHandle> {
        let folder = self.folder.as_ref().ok_or(AssetsError::NoFolderLoaded)?;

        let asset: Asset = asset.into();

        let asset_path = asset.kind() + file_name.into();

        let path = folder.join(asset_path.to_string());

        if !std::fs::exists(&path).is_ok_and(|exists| exists) {
            return Err(anyhow!(std::io::Error::from_raw_os_error(2))
                .context(format!("File {path:?} does not exist.")));
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

    pub fn failed_loads(&self) -> &Vec<(AssetPath, anyhow::Error)> {
        &self.failed_loads
    }

    pub fn load(&mut self, path: &Path) -> Result<()> {
        let ctx = |err: &'static str| format!("Failed while loading directory at {path:?}: {err}");

        if !path.is_dir() {
            return Err(anyhow!(std::io::ErrorKind::NotADirectory))
                .context(ctx("Path is not a dirctory"));
        }

        if !path.exists() {
            std::fs::create_dir(path).context(ctx("Couldn't create directory"))?;
            return Err(anyhow!(std::io::ErrorKind::NotFound));
        }

        let assets = load_dir(path).context(format!("Couldn't load {path:?}"))?;

        let (succeeded, failed): (HashMap<_, _>, _) =
            assets.into_iter().partition(|(_, (_, res))| res.is_ok());

        let failed = failed
            .into_iter()
            .map(|(_, (asset_path, res))| (asset_path, res.unwrap_err()))
            .collect::<Vec<_>>();

        if failed.is_empty() {
            let index: HashMap<u32, AssetPath> = succeeded
                .iter()
                .map(|(id, (asset_path, _))| (*id, asset_path.clone()))
                .collect();

            let assets: HashMap<AssetPath, Asset> = succeeded
                .into_iter()
                .map(|(_, (asset_path, asset))| (asset_path, asset.unwrap()))
                .collect();

            self.index = index;

            self.assets = assets;

            Ok(())
        } else {
            self.failed_loads = failed;

            Err(anyhow!(AssetsError::LoadPartiallyFailed))
        }
    }

    pub fn rename(&mut self, asset_handle: AssetHandle, mut new_name: String) -> Result<()> {
        let old_path = self
            .index
            .get(&asset_handle.0)
            .cloned()
            .ok_or(AssetsError::InvalidAsset)
            .context(format!(
                "Can't rename asset because it doesn't exist: {asset_handle:?}"
            ))?;

        new_name = new_name.trim().to_string();

        if new_name.is_empty() {
            return Err(anyhow!(std::io::ErrorKind::InvalidFilename));
        }

        let extension = old_path.extension();

        if new_name.ends_with(extension) {
            new_name.truncate(new_name.len() - extension.len());
        }

        let new_path = old_path.kind() + (new_name.clone() + extension);

        self.index.insert(asset_handle.0, new_path.clone());

        let folder = self.folder.clone().unwrap();

        let from = folder.clone() + old_path.clone();

        let to = folder + new_path.clone();

        let err_ctx = format!("Couldn't rename {from:?} to {to:?}");

        std::fs::rename(from, to).map_err(|err| anyhow!(err).context(err_ctx.clone()))?;

        match self.write_index() {
            Ok(_) => {
                let asset = self.assets.remove(&old_path).unwrap();
                self.assets.insert(new_path.clone(), asset);

                Ok(())
            }
            Err(err) => Err(anyhow!(err).context(err_ctx)),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (AssetHandle, &AssetPath, &Asset)> {
        self.index.iter().filter_map(|(id, asset_path)| {
            self.assets
                .get(asset_path)
                .map(|asset| (AssetHandle(*id), asset_path, asset))
        })
    }
}
