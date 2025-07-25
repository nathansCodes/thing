use std::path::PathBuf;

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

#[derive(Debug, Clone)]
pub enum IOError {
    DialogClosed,
    IO(io::ErrorKind),
}
