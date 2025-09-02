use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, ErrorKind, Read, Write},
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use file_type::FileType;
use iced::widget::image;
use ron::ser::PrettyConfig;
use thiserror::Error;

use crate::assets::{self, Asset, AssetKind, AssetPath};

pub fn pick_file() -> Result<PathBuf> {
    let file_handle = rfd::FileDialog::new()
        .set_title("Select an Image")
        .add_filter("Image", &["webp", "png", "jpeg", "jpg"])
        .pick_file()
        .ok_or(AssetsError::DialogClosed)?;

    Ok(file_handle)
}

pub fn pick_folder() -> Result<PathBuf> {
    let file_handle = rfd::FileDialog::new()
        .set_title("Open a Folder")
        .pick_folder()
        .ok_or(AssetsError::DialogClosed)?;

    Ok(file_handle)
}

pub fn save(path: PathBuf, data: String) -> Result<()> {
    let mut file = File::create(path)?;

    file.write_all(data.as_bytes())?;

    Ok(())
}

pub fn load(path: PathBuf) -> Result<String> {
    let mut file = File::open(path.join("data.ron"))?;

    let mut data = String::new();

    file.read_to_string(&mut data)?;

    Ok(data)
}

fn new_key<V>(map: &HashMap<u32, V>) -> u32 {
    let keys: Vec<&u32> = map.keys().collect();
    (0..).into_iter().find(|id| !keys.contains(&id)).unwrap()
}

pub fn load_dir(path: PathBuf) -> Result<HashMap<u32, (AssetPath, Asset)>> {
    let index_path = path.join(".index.ron");

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
        .filter_map(|(id, entry)| {
            let path = path.join(entry.to_string());

            let mut file = File::open(path.clone()).ok()?;

            let mut buffer = Vec::new();

            file.read_to_end(&mut buffer).ok()?;

            let reader = BufReader::new(buffer.as_slice());

            let file_type = FileType::try_from_reader(reader).expect("File type not found!");

            let media_types = file_type.media_types();

            let file_name = path.file_name().unwrap().to_string_lossy().to_string();

            if media_types.iter().any(|t| t.contains("image")) {
                Some((
                    id,
                    (
                        AssetPath::new(assets::AssetKind::Image, file_name.clone()),
                        Asset::Image(assets::Image {
                            handle: image::Handle::from_bytes(buffer),
                        }),
                    ),
                ))
            } else {
                None
            }
        })
        .collect();

    Ok(assets)
}

pub fn load_file(path: &Path) -> Result<Asset> {
    let mut file = File::open(path)?;

    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    let reader = BufReader::new(buffer.as_slice());

    let file_type = FileType::try_from_reader(reader).expect("File type not found!");

    if file_type
        .media_types()
        .iter()
        .any(|media| media.starts_with("image"))
    {
        Ok(Asset::Image(assets::Image {
            handle: image::Handle::from_bytes(buffer),
        }))
    } else {
        Err(anyhow!(AssetsError::InvalidAsset))
    }
}

pub fn copy_to_assets_dir(
    assets_folder: Option<PathBuf>,
    path: PathBuf,
) -> Result<(PathBuf, Asset)> {
    let Some(folder) = assets_folder.clone() else {
        return Err(anyhow!(AssetsError::NoFolderLoaded));
    };

    match load_file(&path) {
        Ok(asset) => {
            let file_name = path
                .clone()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            let new_path = folder.join(asset.folder()).join(file_name);

            std::fs::copy(path.clone(), new_path)?;

            Ok((path, asset))
        }
        Err(err) => Err(err),
    }
}

pub fn write_index(index: &HashMap<u32, AssetPath>, assets_folder: Option<PathBuf>) -> Result<()> {
    let Some(assets_folder) = assets_folder else {
        return Err(anyhow!(AssetsError::NoFolderLoaded));
    };

    let parsed_data = ron::ser::to_string_pretty(index, PrettyConfig::new())?;

    let index_path = assets_folder.join(".index.ron");

    let mut index_file = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open(index_path)?;

    index_file.write_all(parsed_data.as_bytes())?;

    Ok(())
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum AssetsError {
    #[error("The dialog was closed.")]
    DialogClosed,
    #[error("File is not a valid asset.")]
    InvalidAsset,
    #[error("Can't complete operation without any folder being loaded.")]
    NoFolderLoaded,
}
