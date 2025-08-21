use std::{collections::HashMap, io, path::PathBuf};

use iced::{
    Alignment, Element,
    Length::Fill,
    Task,
    widget::{button, column, image, row, scrollable, text, text_input},
};
use serde::{Deserialize, Serialize};

use crate::{
    io::{IOError, load_dir},
    style,
    widgets::{dnd::dnd_provider, dropdown, icons},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Image {
    pub name: String,
    pub file_name: String,
    #[serde(skip)]
    #[serde(default = "default_image")]
    pub handle: image::Handle,
}

fn default_image() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../assets/default.png").as_slice())
}

#[derive(Default, Debug, Clone, Copy)]
pub enum ViewMode {
    #[default]
    Thumbnails,
    List,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum AssetKind {
    #[default]
    Image,
}

#[derive(Debug, Clone)]
pub enum Asset {
    Image(Image),
}

#[derive(Default)]
pub struct AssetsData {
    view: AssetKind,
    view_mode: ViewMode,
    view_dropdown_open: bool,
    assets: HashMap<String, Asset>,
    error_state: Option<IOError>,
    filter: String,
    folder: Option<PathBuf>,
}

impl AssetsData {
    pub fn assets(&self) -> &HashMap<String, Asset> {
        &self.assets
    }

    pub fn set_folder(&mut self, folder: PathBuf) {
        self.folder = Some(folder);
    }

    pub fn folder(&self) -> Option<&PathBuf> {
        self.folder.as_ref()
    }

    pub fn get_path(&self, key: impl Into<String>) -> Option<PathBuf> {
        let key = key.into();

        self.assets
            .keys()
            .find(|k| (*k).eq(&key.clone()))
            .and_then(|key| self.folder.clone().map(|folder| folder.join(key)))
    }
}

pub fn update(state: &mut AssetsData, message: AssetsMessage) -> Task<AssetsMessage> {
    match message {
        AssetsMessage::LoadAssets(mut path) => {
            if !path.is_dir() {
                return Task::none();
            }

            let subfolder = match state.view {
                AssetKind::Image => "images",
            };

            path.push(subfolder);

            let create_dir = async |path| -> io::Result<()> { std::fs::create_dir(path) };

            if !path.exists() {
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
        AssetsMessage::LoadCompleted(assets) => {
            state.assets = assets;
            Task::none()
        }
        AssetsMessage::LoadFailed(error) => {
            state.error_state = Some(error);
            Task::none()
        }
        AssetsMessage::OpenAsset(file_name, asset) => {
            Task::done(AssetsMessage::OpenAsset(file_name, asset))
        }
        AssetsMessage::SetPayload(payload) => Task::done(AssetsMessage::SetPayload(payload)),
        AssetsMessage::FilterChanged(text) => {
            state.filter = text;
            Task::none()
        }
        AssetsMessage::ViewChanged(view) => {
            state.view_mode = view;
            Task::none()
        }
        AssetsMessage::ShowHideDropdown => {
            state.view_dropdown_open = !state.view_dropdown_open;
            Task::none()
        }
    }
}

fn image_item<'a>(
    i: usize,
    key: impl Into<String>,
    path: PathBuf,
    view: ViewMode,
    img: &'a Image,
) -> Element<'a, AssetsMessage, iced::Theme, iced::Renderer> {
    dnd_provider(
        AssetsMessage::SetPayload,
        crate::Draggable::Asset(Asset::Image(img.clone())),
        match view {
            ViewMode::Thumbnails => button(
                column![
                    image(path.clone())
                        .height(100.0)
                        .width(100.0)
                        .filter_method(image::FilterMethod::Nearest),
                    text(img.file_name.clone()).width(100.0).center()
                ]
                .spacing(5.0)
                .padding(5.0),
            )
            .padding(0)
            .style(style::list_item(false))
            .on_press(AssetsMessage::OpenAsset(
                key.into(),
                Asset::Image(img.clone()),
            )),
            ViewMode::List => button(
                row![
                    image(path.clone())
                        .height(30)
                        .width(30)
                        .filter_method(image::FilterMethod::Nearest),
                    text(img.file_name.clone())
                ]
                .width(Fill)
                .height(40)
                .spacing(10)
                .padding(5)
                .align_y(Alignment::Center),
            )
            .on_press(AssetsMessage::OpenAsset(
                key.into(),
                Asset::Image(img.clone()),
            ))
            .padding(0)
            .style(style::list_item(i % 2 == 0)),
        },
    )
}

pub fn view(state: &AssetsData) -> Element<'_, AssetsMessage> {
    let content = match state.view {
        AssetKind::Image => {
            let mut images: Vec<_> = state
                .assets
                .iter()
                .filter(|(file_name, _)| {
                    file_name
                        .to_lowercase()
                        .contains(&state.filter.to_lowercase())
                })
                .collect();

            images.sort_by(|a, b| a.0.cmp(b.0));

            let images = images
                .into_iter()
                .enumerate()
                .filter_map(|(i, (key, asset))| {
                    state.get_path(key).map(|path| match asset {
                        Asset::Image(image) => {
                            image_item(i, key, path.clone(), state.view_mode, image)
                        }
                    })
                });

            let layout = match state.view_mode {
                ViewMode::Thumbnails => Element::from(row(images).spacing(5).width(Fill).wrap()),
                ViewMode::List => Element::from(column(images).spacing(2).width(Fill)),
            };

            scrollable(layout).style(style::scrollable)
        }
    };

    let dropdown = dropdown(
        state.view_dropdown_open,
        AssetsMessage::ShowHideDropdown,
        match state.view_mode {
            ViewMode::Thumbnails => icons::thumbnails(),
            ViewMode::List => icons::list(),
        },
        [
            (
                icons::THUMBNAILS,
                "Thumbnails",
                AssetsMessage::ViewChanged(ViewMode::Thumbnails),
            ),
            (
                icons::LIST,
                "List",
                AssetsMessage::ViewChanged(ViewMode::List),
            ),
        ]
        .into_iter(),
    );

    column![
        row![
            text_input("Search...", &state.filter)
                .on_input(AssetsMessage::FilterChanged)
                .icon(text_input::Icon {
                    font: icons::ICON_FONT,
                    code_point: icons::SEARCH,
                    size: None,
                    spacing: 4.0,
                    side: text_input::Side::Left
                })
                .style(style::text_input),
            dropdown
        ]
        .spacing(4.0),
        content,
    ]
    .spacing(4.0)
    .into()
}

#[derive(Clone, Debug)]
pub enum AssetsMessage {
    LoadAssets(PathBuf),
    LoadCompleted(HashMap<String, Asset>),
    LoadFailed(IOError),
    OpenAsset(String, Asset),
    SetPayload(Option<crate::Draggable>),
    FilterChanged(String),
    ViewChanged(ViewMode),
    ShowHideDropdown,
}
