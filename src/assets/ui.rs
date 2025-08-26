use std::io;

use iced::{
    Alignment, Element,
    Length::Fill,
    Task,
    widget::{self, button, column, row, scrollable, text, text_input},
};

use crate::{
    assets::{Asset, AssetHandle, AssetKind, AssetsData, AssetsMessage, Image, ViewMode},
    io::{IOError, load_dir},
    style,
    widgets::{dnd::dnd_provider, dropdown, icons},
};

pub fn update(state: &mut AssetsData, message: AssetsMessage) -> Task<AssetsMessage> {
    match message {
        AssetsMessage::LoadAssets(path) => {
            if !path.is_dir() {
                return Task::none();
            }

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
            println!("{error:?}");
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
                            .to_string()
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
