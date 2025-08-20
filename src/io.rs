use std::{
    io::{Read, Write},
    path::PathBuf,
};

use iced::futures::io;

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

pub async fn load_dir(path: PathBuf) -> Result<Vec<PathBuf>, IOError> {
    let dir = std::fs::read_dir(path).map_err(|err| IOError::IO(err.kind()))?;

    let files: Vec<PathBuf> = dir
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();

    Ok(files)
}

#[derive(Debug, Clone)]
pub enum IOError {
    DialogClosed,
    IO(io::ErrorKind),
    DeserializationFailed(toml::de::Error),
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
