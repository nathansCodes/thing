use std::{ffi::OsStr, io, path::PathBuf};

use iced::{
    Alignment, Element,
    Length::Fill,
    Task,
    widget::{button, column, image, row, scrollable, text, text_input},
};

use crate::{
    io::{IOError, load_dir},
    style,
    widgets::{dnd::dnd_provider, dropdown, icons},
};

#[derive(Default, Debug, Clone, Copy)]
pub enum View {
    #[default]
    Thumbnails,
    List,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum AssetType {
    #[default]
    Image,
}

#[derive(Default)]
pub struct AssetsPane {
    current_assets: AssetType,
    view: View,
    view_dropdown_open: bool,
    files: Vec<PathBuf>,
    error_state: Option<IOError>,
    filter: String,
}

pub fn update(state: &mut AssetsPane, message: AssetsMessage) -> Task<AssetsMessage> {
    match message {
        AssetsMessage::LoadAssets(mut path) => {
            if !path.is_dir() {
                return Task::none();
            }

            let subfolder = match state.current_assets {
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
        AssetsMessage::FilterChanged(text) => {
            state.filter = text;
            Task::none()
        }
        AssetsMessage::ViewChanged(view) => {
            state.view = view;
            Task::none()
        }
        AssetsMessage::ShowHideDropdown => {
            state.view_dropdown_open = !state.view_dropdown_open;
            Task::none()
        }
    }
}

pub fn view(state: &AssetsPane) -> Element<'_, AssetsMessage> {
    let content = match state.current_assets {
        AssetType::Image => {
            let images = state
                .files
                .iter()
                .filter(|path| {
                    path.file_name()
                        .unwrap_or(OsStr::new(""))
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&state.filter.to_lowercase())
                })
                .enumerate()
                .map(|(i, path)| {
                    dnd_provider(
                        AssetsMessage::SetPayload,
                        crate::Draggable::ImageAsset(path.clone()),
                        match state.view {
                            View::Thumbnails => button(
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
                            )
                            .padding(0)
                            .style(style::list_item(false))
                            .on_press(AssetsMessage::OpenAsset(state.current_assets, path.clone())),
                            View::List => button(
                                row![
                                    image(path)
                                        .height(30)
                                        .width(30)
                                        .filter_method(image::FilterMethod::Nearest),
                                    text(
                                        path.file_name()
                                            .map(|os_str| os_str.to_string_lossy())
                                            .unwrap_or("abcd".into()),
                                    )
                                ]
                                .width(Fill)
                                .height(40)
                                .spacing(10)
                                .padding(5)
                                .align_y(Alignment::Center),
                            )
                            .on_press(AssetsMessage::OpenAsset(state.current_assets, path.clone()))
                            .padding(0)
                            .style(style::list_item(i % 2 == 0)),
                        },
                    )
                });

            let layout = match state.view {
                View::Thumbnails => Element::from(row(images).spacing(5).width(Fill).wrap()),
                View::List => Element::from(column(images).spacing(2).width(Fill)),
            };

            scrollable(layout).style(style::scrollable)
        }
    };

    let dropdown = dropdown(
        state.view_dropdown_open,
        AssetsMessage::ShowHideDropdown,
        match state.view {
            View::Thumbnails => icons::thumbnails(),
            View::List => icons::list(),
        },
        [
            (
                icons::THUMBNAILS,
                "Thumbnails",
                AssetsMessage::ViewChanged(View::Thumbnails),
            ),
            (icons::LIST, "List", AssetsMessage::ViewChanged(View::List)),
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
    LoadCompleted(Vec<PathBuf>),
    LoadFailed(IOError),
    OpenAsset(AssetType, PathBuf),
    SetPayload(Option<crate::Draggable>),
    FilterChanged(String),
    ViewChanged(View),
    ShowHideDropdown,
}
