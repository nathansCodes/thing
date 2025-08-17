use std::{io, path::PathBuf};

use iced::{
    Element,
    Length::Fill,
    Task,
    mouse::Interaction,
    widget::{column, container, image, mouse_area, row, scrollable, text},
};

use crate::{io::IOError, widgets::dnd::dnd_provider};

#[derive(Default, Debug, Clone, Copy)]
pub enum AssetType {
    #[default]
    Image,
}

#[derive(Default)]
pub struct AssetsPane {
    view: AssetType,
    files: Vec<PathBuf>,
    error_state: Option<IOError>,
}

pub fn update(state: &mut AssetsPane, message: AssetsMessage) -> Task<AssetsMessage> {
    match message {
        AssetsMessage::LoadAssets(mut path) => {
            if !path.is_dir() {
                return Task::none();
            }

            let subfolder = match state.view {
                AssetType::Image => "images",
            };

            path.push(PathBuf::from(subfolder));

            let create_dir = async |path| -> io::Result<()> { std::fs::create_dir(path) };

            if !path.exists() {
                println!("/images subdirectory not found");
                Task::perform(create_dir(path.clone()), move |result| match result {
                    Ok(_) => {
                        let mut path = path.clone();
                        path.pop();
                        AssetsMessage::LoadAssets(path)
                    }
                    Err(err) => AssetsMessage::LoadFailed(IOError::IO(err.kind())),
                })
            } else if !path.is_dir() {
                Task::done(AssetsMessage::LoadFailed(IOError::IO(
                    io::ErrorKind::AlreadyExists,
                )))
            } else {
                Task::perform(load_dir(path), |files_maybe| match files_maybe {
                    Ok(files) => AssetsMessage::LoadCompleted(files),
                    Err(err) => AssetsMessage::LoadFailed(err),
                })
            }
        }
        AssetsMessage::LoadCompleted(path_bufs) => {
            state.files = path_bufs;
            Task::none()
        }
        AssetsMessage::LoadFailed(error) => {
            state.error_state = Some(error);
            Task::none()
        }
        AssetsMessage::OpenAsset(asset_type, path) => {
            Task::done(AssetsMessage::OpenAsset(asset_type, path))
        }
        AssetsMessage::SetPayload(payload) => Task::done(AssetsMessage::SetPayload(payload)),
    }
}

pub fn view(state: &AssetsPane) -> Element<'_, AssetsMessage> {
    match state.view {
        AssetType::Image => scrollable(
            row(state.files.iter().map(|path| {
                let img = dnd_provider(
                    AssetsMessage::SetPayload,
                    crate::Draggable::ImageAsset(path.clone()),
                    mouse_area(container(
                        column![
                            image(path)
                                .height(100.0)
                                .width(100.0)
                                .filter_method(image::FilterMethod::Nearest),
                            text(
                                path.file_name()
                                    .map(|os_str| os_str.to_string_lossy())
                                    .unwrap_or("abcd".into()),
                            )
                            .width(100.0)
                            .center()
                        ]
                        .spacing(5.0)
                        .padding(5.0),
                    ))
                    .on_release(AssetsMessage::OpenAsset(state.view, path.clone()))
                    .interaction(Interaction::Pointer),
                );
                Element::from(img)
            }))
            .spacing(5.0)
            .width(Fill)
            .wrap(),
        )
        .into(),
    }
}

async fn load_dir(path: PathBuf) -> Result<Vec<PathBuf>, IOError> {
    let dir = std::fs::read_dir(path).map_err(|err| IOError::IO(err.kind()))?;

    let files: Vec<PathBuf> = dir
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();

    Ok(files)
}

#[derive(Clone, Debug)]
pub enum AssetsMessage {
    LoadAssets(PathBuf),
    LoadCompleted(Vec<PathBuf>),
    LoadFailed(IOError),
    OpenAsset(AssetType, PathBuf),
    SetPayload(Option<crate::Draggable>),
}
