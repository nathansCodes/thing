pub mod image;

pub use image::Image;

use std::{collections::HashMap, io, ops::Index, path::PathBuf};

use iced::{
    Alignment, Element,
    Length::Fill,
    Task,
    widget::{self, button, column, row, scrollable, text, text_input},
};
use serde::{Deserialize, Serialize};

use crate::{
    io::{IOError, load_dir},
    style,
    widgets::{dnd::dnd_provider, dropdown, icons},
};

#[derive(Debug, Clone)]
pub enum Asset {
    Image(Image),
}

impl Asset {
    pub fn folder(&self) -> String {
        match self {
            Self::Image(_) => "images".to_string(),
        }
    }

    pub fn kind(&self) -> AssetKind {
        match self {
            Self::Image(_) => AssetKind::Image,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AssetHandle(u32);

#[derive(Default, Debug, Clone, Copy)]
pub enum AssetKind {
    #[default]
    Image,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum ViewMode {
    #[default]
    Thumbnails,
    List,
}

#[derive(Default)]
pub struct AssetsData {
    view: AssetKind,
    view_mode: ViewMode,
    view_dropdown_open: bool,
    assets: HashMap<String, Asset>,
    index: HashMap<u32, String>,
    error_state: Option<IOError>,
    filter: String,
    folder: Option<PathBuf>,
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

    pub fn handle(&self, asset_path: String) -> Option<AssetHandle> {
        self.assets.get(&asset_path).and_then(|_| {
            self.index
                .iter()
                .find_map(|(id, path)| path.eq(&asset_path).then_some(AssetHandle(*id)))
        })
    }

    pub fn add(
        &mut self,
        file_name: impl Into<String>,
        asset: impl Into<Asset>,
    ) -> Result<AssetHandle, IOError> {
        let folder = self.folder.as_ref().ok_or(IOError::NoFolderLoaded)?;

        let asset: Asset = asset.into();

        let mut asset_path = asset.folder() + "/";

        asset_path.push_str(&file_name.into());

        let path = folder.join(&asset_path);

        if !std::fs::exists(&path).is_ok_and(|exists| exists) {
            return Err(IOError::OSError(2));
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
            for (id, (asset_path, asset)) in assets.into_iter() {
                state.index.insert(id, asset_path.clone());
                state.assets.insert(asset_path, asset);
            }

            Task::none()
        }
        AssetsMessage::LoadFailed(error) => {
            state.error_state = Some(error);
            Task::none()
        }
        AssetsMessage::OpenAsset(handle) => Task::done(AssetsMessage::OpenAsset(handle)),
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
    handle: AssetHandle,
    view: ViewMode,
    img: &'a Image,
) -> Element<'a, AssetsMessage, iced::Theme, iced::Renderer> {
    dnd_provider(
        AssetsMessage::SetPayload,
        crate::Draggable::Asset(handle),
        match view {
            ViewMode::Thumbnails => button(
                column![
                    widget::image(&img.handle)
                        .height(100.0)
                        .width(100.0)
                        .filter_method(widget::image::FilterMethod::Nearest),
                    text(img.file_name.clone()).width(100.0).center()
                ]
                .spacing(5.0)
                .padding(5.0),
            )
            .padding(0)
            .style(style::list_item(false))
            .on_press(AssetsMessage::OpenAsset(handle)),
            ViewMode::List => button(
                row![
                    widget::image(&img.handle)
                        .height(30)
                        .width(30)
                        .filter_method(widget::image::FilterMethod::Nearest),
                    text(img.file_name.clone())
                ]
                .width(Fill)
                .height(40)
                .spacing(10)
                .padding(5)
                .align_y(Alignment::Center),
            )
            .on_press(AssetsMessage::OpenAsset(handle))
            .padding(0)
            .style(style::list_item(i % 2 == 0)),
        },
    )
}

pub fn view(state: &AssetsData) -> Element<'_, AssetsMessage> {
    let content = match state.view {
        AssetKind::Image => {
            let mut images: Vec<_> = state
                .index
                .iter()
                .filter_map(|(id, asset_path)| {
                    state.assets.get(asset_path).and_then(|asset| {
                        asset_path
                            .to_lowercase()
                            .contains(&state.filter.to_lowercase())
                            .then_some(())
                            // WARNING: change to .and_then when adding new types of assets
                            .map(|_| match asset {
                                Asset::Image(img) => (*id, img),
                            })
                    })
                })
                .collect();

            images.sort_by(|a, b| a.0.cmp(&b.0));

            let images = images
                .into_iter()
                .enumerate()
                .map(|(i, (id, img))| image_item(i, AssetHandle(id), state.view_mode, img));

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
    LoadCompleted(HashMap<u32, (String, Asset)>),
    LoadFailed(IOError),
    OpenAsset(AssetHandle),
    SetPayload(Option<crate::Draggable>),
    FilterChanged(String),
    ViewChanged(ViewMode),
    ShowHideDropdown,
}
