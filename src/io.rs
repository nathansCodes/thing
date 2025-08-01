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

pub async fn load(mut path: PathBuf) -> Result<String, IOError> {
    path.push("data.toml");

    let mut file = if std::fs::exists(path.clone())? {
        std::fs::File::open(path.clone())?
    } else {
        std::fs::File::create(path)?
    };

    println!("{file:?}");

    let mut data = String::new();

    file.read_to_string(&mut data)?;

    Ok(data)
}

#[derive(Debug, Clone)]
pub enum IOError {
    DialogClosed,
    IO(io::ErrorKind),
    DeserializationFailed(toml::de::Error),
}

impl From<std::io::Error> for IOError {
    fn from(value: std::io::Error) -> Self {
        println!("{value:?}");
        Self::IO(value.kind())
    }
}

impl From<toml::de::Error> for IOError {
    fn from(value: toml::de::Error) -> Self {
        Self::DeserializationFailed(value)
    }
}
