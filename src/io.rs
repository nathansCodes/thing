use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, ErrorKind, Read, Write},
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
};

use file_type::FileType;
use iced::{futures::io, widget::image};
use ron::ser::PrettyConfig;
use thiserror::Error;

use crate::assets::{self, Asset, AssetKind, AssetPath};

pub async fn pick_file() -> Result<PathBuf, IOError> {
    let file_handle = rfd::AsyncFileDialog::new()
        .set_title("Select an Image")
        .add_filter("Image", &["webp", "png", "jpeg", "jpg"])
        .pick_file()
        .await
        .ok_or(IOError::DialogClosed)?;

    Ok(file_handle.path().to_path_buf())
}

pub async fn pick_folder() -> Result<PathBuf, IOError> {
    let file_handle = rfd::AsyncFileDialog::new()
        .set_title("Open a Folder")
        .pick_folder()
        .await
        .ok_or(IOError::DialogClosed)?;

    Ok(file_handle.path().to_path_buf())
}

pub async fn save(path: PathBuf, data: String) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;

    file.write_all(data.as_bytes())?;

    Ok(())
}

pub async fn load(path: PathBuf) -> Result<String, IOError> {
    let mut file = File::open(path.join("data.ron"))?;

    let mut data = String::new();

    file.read_to_string(&mut data)?;

    Ok(data)
}

fn new_key<V>(map: &HashMap<u32, V>) -> u32 {
    let keys: Vec<&u32> = map.keys().collect();
    (0..).into_iter().find(|id| !keys.contains(&id)).unwrap()
}

pub async fn load_dir(path: PathBuf) -> Result<HashMap<u32, (AssetPath, Asset)>, IOError> {
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
            _ => return Err(IOError::from(err)),
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
                            file_name,
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

pub async fn load_file(path: &Path) -> Result<Asset, IOError> {
    let mut file = File::open(path)?;

    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    let reader = BufReader::new(buffer.as_slice());

    let file_type = FileType::try_from_reader(reader).expect("File type not found!");

    let file_name = path.file_name().unwrap().to_string_lossy().to_string();

    if file_type
        .media_types()
        .iter()
        .any(|media| media.starts_with("image"))
    {
        Ok(Asset::Image(assets::Image {
            file_name,
            handle: image::Handle::from_bytes(buffer),
        }))
    } else {
        Err(IOError::InvalidAsset)
    }
}

pub async fn copy_to_assets_dir(
    assets_folder: Option<PathBuf>,
    path: PathBuf,
) -> Result<(PathBuf, Asset), IOError> {
    let Some(folder) = assets_folder.clone() else {
        return Err(IOError::NoFolderLoaded);
    };

    match load_file(&path).await {
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

#[derive(Error, Debug, Clone)]
pub enum IOError {
    #[error("The dialog was closed.")]
    DialogClosed,
    #[error("IO error occurred: {0}")]
    IO(io::ErrorKind),
    #[error("OS error {0} occurred {err}", err = io::Error::from_raw_os_error(*.0).kind())]
    OSError(i32),
    #[error("Deserialization failed at {pos}: {err}", pos = .0.position, err = .0.code)]
    DeserializationFailed(ron::de::SpannedError),
    #[error("File is not a valid asset.")]
    InvalidAsset,
    #[error("Can't complete operation without any folder being loaded.")]
    NoFolderLoaded,
}

impl From<std::io::Error> for IOError {
    fn from(value: std::io::Error) -> Self {
        match value.raw_os_error() {
            Some(err) => Self::OSError(err),
            None => Self::IO(value.kind()),
        }
    }
}

impl From<ron::de::SpannedError> for IOError {
    fn from(value: ron::de::SpannedError) -> Self {
        Self::DeserializationFailed(value)
    }
}
