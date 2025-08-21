use std::{
    collections::HashMap,
    io::{BufReader, Read, Write},
    path::PathBuf,
};

use file_type::FileType;
use iced::{futures::io, widget::image};

use crate::assets::{self, Asset};

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
    let mut file = std::fs::File::create(path)?;

    file.write_all(data.as_bytes())?;

    Ok(())
}

pub async fn load(path: PathBuf) -> Result<String, IOError> {
    let mut file = std::fs::File::open(path.join("data.toml"))?;

    let mut data = String::new();

    file.read_to_string(&mut data)?;

    Ok(data)
}

pub async fn load_dir(path: PathBuf) -> Result<HashMap<String, Asset>, IOError> {
    let dir = std::fs::read_dir(path).map_err(|err| IOError::IO(err.kind()))?;

    let assets: HashMap<String, Asset> = dir
        .filter_map(|entry| {
            entry.ok().and_then(|entry| {
                let path = entry.path();

                let mut file = std::fs::File::open(path.clone()).ok()?;

                let mut buffer = Vec::new();

                file.read_to_end(&mut buffer).ok()?;

                let reader = BufReader::new(buffer.as_slice());

                let file_type = FileType::try_from_reader(reader).expect("File type not found!");

                let file_name = path.file_name().unwrap().to_string_lossy().to_string();

                let mut key = path
                    .parent()
                    .and_then(|path| {
                        path.file_name()
                            .map(|file_name| file_name.to_string_lossy().to_string())
                    })
                    .unwrap_or("".to_string())
                    + "/";

                key.push_str(&file_name);

                file_type
                    .media_types()
                    .iter()
                    .any(|t| t.contains("image"))
                    .then_some((
                        key,
                        Asset::Image(assets::Image {
                            name: path.file_stem().unwrap().to_string_lossy().to_string(),
                            file_name,
                            handle: image::Handle::from_bytes(buffer),
                        }),
                    ))
            })
        })
        .collect();

    Ok(assets)
}

pub async fn load_file(path: PathBuf) -> Result<Asset, IOError> {
    let mut file = std::fs::File::open(path.clone())?;

    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    let reader = BufReader::new(buffer.as_slice());

    let file_type = FileType::try_from_reader(reader).expect("File type not found!");

    let file_name = path.file_name().unwrap().to_string_lossy().to_string();

    if file_type.media_types().contains(&"image") {
        Ok(Asset::Image(assets::Image {
            name: path.file_stem().unwrap().to_string_lossy().to_string(),
            file_name,
            handle: image::Handle::from_bytes(buffer),
        }))
    } else {
        Err(IOError::InvalidAsset)
    }
}

#[derive(Debug, Clone)]
pub enum IOError {
    DialogClosed,
    IO(io::ErrorKind),
    DeserializationFailed(toml::de::Error),
    InvalidAsset,
}

impl From<std::io::Error> for IOError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value.kind())
    }
}

impl From<toml::de::Error> for IOError {
    fn from(value: toml::de::Error) -> Self {
        Self::DeserializationFailed(value)
    }
}
