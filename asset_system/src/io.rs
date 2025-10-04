use crate::{AsBytes, Asset, AssetKind, AssetPath, AssetsData};

use anyhow::{Result, anyhow};
use file_type::FileType;
use iced::advanced::{graphics::image::image_rs::ImageFormat, image};
use ron::ser::PrettyConfig;
use thiserror::Error;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, ErrorKind, Read, Write},
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
};

pub const DATA_FILE: &str = "data.ron";
pub const INDEX_FILE: &str = ".index.ron";

impl AssetsData {
    pub fn copy_to_assets_dir(&self, path: &Path) -> Result<(PathBuf, Asset)> {
        let Some(folder) = self.folder() else {
            return Err(anyhow!(AssetsError::NoFolderLoaded));
        };

        match load_file(path) {
            Ok(asset) => {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();

                let new_path = folder.join(asset.folder()).join(file_name);

                std::fs::copy(path, &new_path)?;

                Ok((new_path, asset))
            }
            Err(err) => Err(err),
        }
    }

    pub fn write_index(&self) -> Result<()> {
        let Some(assets_folder) = self.folder() else {
            return Err(anyhow!(AssetsError::NoFolderLoaded));
        };

        let parsed_data = ron::ser::to_string_pretty(&self.index, PrettyConfig::new())?;

        let index_path = assets_folder.join(INDEX_FILE);

        let mut index_file = File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(index_path)?;

        index_file.write_all(parsed_data.as_bytes())?;

        Ok(())
    }

    pub fn write_assets(&self) -> Result<Vec<(u32, anyhow::Error)>> {
        let Some(folder) = self.folder() else {
            return Err(anyhow!(AssetsError::NoFolderLoaded));
        };

        let results = self.iter().filter_map(|(handle, asset_path, asset)| {
            let result: Result<()> = (|| {
                let bytes = asset.as_bytes()?;

                let path = folder + asset_path;

                let mut file = File::options()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(path)?;

                file.write_all(&bytes)?;

                Ok(())
            })();

            result.err().map(|err| {
                (
                    handle.0,
                    err.context(format!("Failed to save {asset_path} ({})", handle.0)),
                )
            })
        });

        Ok(results.collect())
    }
}

pub fn load(path: &Path) -> Result<String> {
    let mut file = File::open(path.join(DATA_FILE))?;

    let mut data = String::new();

    file.read_to_string(&mut data)?;

    Ok(data)
}

fn asset_from_bytes<R: std::io::Read>(mut reader: R, path: &Path) -> Result<Asset> {
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)?;

    let file_type = FileType::from_bytes(&buffer);

    let file_name = path.file_name().unwrap().to_string_lossy().to_string();

    let mime_types = file_type.media_types();

    let is_mime_type = |mime| mime_types.iter().find(|t| t.starts_with(mime));

    if let Some(mime_type) = is_mime_type("image")
        && let Some(format) = ImageFormat::from_mime_type(mime_type)
    {
        let img = super::Image::new(format, image::Handle::from_bytes(buffer));

        Ok(Asset::Image(img))
    } else if is_mime_type("text/plain").is_some() && file_name.ends_with(".chara.ron") {
        ron::de::from_bytes(&buffer)
            .map(Asset::Character)
            .map_err(|err| anyhow!(err))
    } else {
        println!(
            "invalid: {path:?}; mime: {:?}; file_name: {file_name}",
            mime_types
        );
        Err(anyhow!(AssetsError::InvalidAsset))
    }
}

pub fn load_file(path: &Path) -> Result<Asset> {
    let mut file = File::open(path)?;

    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    let reader = BufReader::new(buffer.as_slice());

    asset_from_bytes(reader, path)
}

pub fn load_dir(path: &Path) -> Result<HashMap<u32, (AssetPath, Result<Asset>)>> {
    let index_path = path.join(INDEX_FILE);

    let mut index_file = match File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(index_path.clone())
    {
        Ok(index_file) => index_file,
        Err(err) => match err.kind() {
            ErrorKind::NotFound => File::create_new(index_path)?,
            _ => return Err(anyhow!(err)),
        },
    };

    let mut index_file_contents = String::new();

    index_file.read_to_string(&mut index_file_contents)?;

    let mut index: HashMap<u32, AssetPath> = if index_file_contents.trim().is_empty() {
        HashMap::default()
    } else {
        ron::de::from_str(&index_file_contents)?
    };

    for kind in AssetKind::all() {
        let dir: Vec<_> = std::fs::read_dir(path.join(kind.folder()))?
            .filter_map(|entry| entry.ok())
            .collect();

        let current_index_size = index
            .iter()
            .filter(|(_, path)| path.kind() == *kind)
            .count();

        if current_index_size < dir.len() {
            for entry in dir {
                let file_name = entry.file_name().to_string_lossy().to_string();

                let asset_path = AssetPath::new(*kind, file_name);

                if !index.values().any(|path| path == &asset_path)
                    && let Ok(asset_path) = AssetPath::try_from(asset_path.to_string().as_str())
                {
                    let id = new_key(&index);
                    index.insert(id, asset_path);
                }
            }

            if let Ok(index_str) = ron::ser::to_string_pretty(&index, PrettyConfig::new()) {
                if let Err(err) = index_file.set_len(0) {
                    eprintln!("{err}");
                };
                match index_file.write_all_at(index_str.as_bytes(), 0) {
                    Ok(_) => (),
                    Err(err) => {
                        eprintln!("{err}");
                    }
                }
            }
        }
    }

    let assets = index
        .into_iter()
        .map(|(id, entry)| {
            let path = path.join(entry.to_string());

            let mut file = match File::open(path.clone()) {
                Ok(file) => file,
                Err(err) => return (id, (entry, Err(anyhow!(err)))),
            };

            let mut buffer = Vec::new();

            if let Err(err) = file.read_to_end(&mut buffer) {
                return (id, (entry, Err(anyhow!(err))));
            }

            let reader = BufReader::new(buffer.as_slice());

            (id, (entry, asset_from_bytes(reader, &path)))
        })
        .collect();

    println!("{assets:#?}");

    Ok(assets)
}

pub fn save(path: &Path, data: String) -> Result<()> {
    let file_path = path.join(DATA_FILE);

    let mut file = File::create(file_path)?;

    file.write_all(data.as_bytes())?;

    Ok(())
}

fn new_key<V>(map: &HashMap<u32, V>) -> u32 {
    let keys: Vec<&u32> = map.keys().collect();
    (0..).into_iter().find(|id| !keys.contains(&id)).unwrap()
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum AssetsError {
    #[error("The dialog was closed.")]
    DialogClosed,
    #[error("File is not a valid asset.")]
    InvalidAsset,
    #[error("Can't complete operation without any folder being loaded.")]
    NoFolderLoaded,
    #[error("Loading of assets partially failed.")]
    LoadPartiallyFailed,
}
