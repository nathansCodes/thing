use std::io;

use crate::{
    assets::{
        Asset, AssetHandle, AssetKind, AssetPath, AssetsData, AssetsMessage, Image, ViewMode,
    },
    io::{AssetsError, load_dir, write_index},
    style,
    widgets::{self, dnd::dnd_provider, dropdown, icons},
};

use iced::{
    Alignment, Element,
    Length::{Fill, Shrink},
    Task,
    widget::{
        self, button, column, container, horizontal_space, responsive, row, scrollable, text,
        text_input,
    },
};
use iced_aw::ContextMenu;

use anyhow::anyhow;

pub fn update(state: &mut AssetsData, message: AssetsMessage) -> Task<AssetsMessage> {
    match message {
        AssetsMessage::LoadAssets(path) => {
            if !path.is_dir() {
                return Task::none();
            }

            if !path.exists()
                && let Err(err) = std::fs::create_dir(path.clone())
            {
                state.last_error = Some(anyhow!(err).context(format!("Couldn't create {path:?}")));
                return Task::done(AssetsMessage::LoadFailed);
            }

            let message = if !path.is_dir() {
                state.last_error = Some(
                    anyhow!(io::Error::from(io::ErrorKind::AlreadyExists))
                        .context(format!("Couldn't create {path:?}")),
                );
                AssetsMessage::LoadFailed
            } else {
                match load_dir(path.clone()) {
                    Ok(files) => AssetsMessage::LoadCompleted(path, files),
                    Err(err) => {
                        state.last_error =
                            Some(anyhow!(err).context(format!("Couldn't load {path:?}")));

                        AssetsMessage::LoadFailed
                    }
                }
            };

            Task::done(message)
        }
        AssetsMessage::LoadCompleted(_, assets) => {
            for (id, (asset_path, asset)) in assets.into_iter() {
                state.index.insert(id, asset_path.clone());
                state.assets.insert(asset_path, asset);
            }

            Task::none()
        }
        AssetsMessage::LoadFailed => {
            if let Some(err) = &state.last_error {
                println!("{err:#}");
            }

            Task::none()
        }
        AssetsMessage::OpenAsset(handle) => Task::done(AssetsMessage::OpenAsset(handle)),
        AssetsMessage::SetPayload(payload) => Task::done(AssetsMessage::SetPayload(payload)),
        AssetsMessage::QueryChanged(text) => {
            state.query = text;
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
        AssetsMessage::SetRenameInput(val) => {
            state.renaming = val;
            Task::none()
        }
        AssetsMessage::RenameAsset => {
            let Some((handle, new_name)) = state.renaming.take() else {
                return Task::none();
            };

            let Some(old_path) = state.index.get(&handle.0).cloned() else {
                return Task::none();
            };

            let new_path = AssetPath::new(old_path.kind(), new_name.clone());

            state.index.insert(handle.0, new_path.clone());

            let folder = state.folder.clone().unwrap();

            let from = folder.clone() + old_path.clone();

            let to = folder + new_path.clone();

            let err_ctx = format!("Couldn't rename {from:?} to {to:?}");

            if let Err(err) = std::fs::rename(from, to) {
                state.last_error = Some(anyhow!(err).context(err_ctx));

                return Task::done(AssetsMessage::RenameAssetFailed(handle));
            };

            let res = write_index(&state.index, state.folder.clone());

            match res {
                Ok(_) => {
                    let asset = state.assets.remove(&old_path).unwrap();
                    state.assets.insert(new_path.clone(), asset);

                    Task::none()
                }
                Err(err) => {
                    if let Some(err) = err.downcast_ref::<AssetsError>() {
                        state.last_error = Some(anyhow!(err.clone()).context(err_ctx));

                        return Task::done(AssetsMessage::RenameAssetFailed(handle));
                    }
                    Task::none()
                }
            }
        }
        AssetsMessage::RenameAssetFailed(..) => Task::none(),
    }
}

fn image_item<'a>(
    i: usize,
    handle: AssetHandle,
    path: &'a AssetPath,
    state: &'a AssetsData,
    img: &'a Image,
) -> Element<'a, AssetsMessage, iced::Theme, iced::Renderer> {
    let rename_input = state.renaming.as_ref().map(|(rn_handle, input)| {
        text_input("Rename...", input.as_str())
            .on_input(|input| AssetsMessage::SetRenameInput(Some((*rn_handle, input))))
            .on_submit(AssetsMessage::RenameAsset)
    });

    let file_name = path.name();

    dnd_provider(
        AssetsMessage::SetPayload,
        crate::Draggable::Asset(handle),
        match state.view_mode {
            ViewMode::Thumbnails => button(
                column![
                    widget::image(&img.handle)
                        .height(100.0)
                        .width(100.0)
                        .filter_method(widget::image::FilterMethod::Nearest),
                    rename_input
                        .map(|ri| Element::from(ri.width(100.0).align_x(Alignment::Center)))
                        .unwrap_or(text(file_name).width(100.0).center().into())
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
                    rename_input
                        .map(Element::from)
                        .unwrap_or(text(file_name).into())
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
                            .contains(&state.query().to_lowercase())
                            .then_some(asset_path)
                            // WARNING: change to .and_then when adding new types of assets
                            .map(|path| match asset {
                                Asset::Image(img) => (*id, path, img),
                            })
                    })
                })
                .collect();

            images.sort_by(|a, b| a.0.cmp(&b.0));

            let images = images.into_iter().enumerate().map(|(i, (id, path, img))| {
                let handle = AssetHandle(id);
                let img_element = image_item(i, handle, path, state, img);

                ContextMenu::new(img_element, move || {
                    container(column![widgets::menu_button(
                        "Rename",
                        AssetsMessage::SetRenameInput(Some((handle, path.to_string())))
                    )])
                    .padding(4)
                    .style(style::dropdown)
                    .into()
                })
                .into()
            });

            let layout = match state.view_mode {
                ViewMode::Thumbnails => Element::from(row(images).spacing(5).width(Fill).wrap()),
                ViewMode::List => Element::from(column(images).spacing(2).width(Fill)),
            };

            scrollable(layout).style(style::scrollable)
        }
    };

    let top_row = responsive(move |size| {
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

        let search_button = button(icons::search().center())
            .width(30)
            .on_press(AssetsMessage::QueryChanged(Some("".to_string())))
            .style(style::menu_button);

        let top_right_row = row![dropdown].spacing(4.0);

        if let Some(query) = &state.query {
            let search_bar = text_input("Search...", query)
                .on_input(|input| AssetsMessage::QueryChanged(Some(input)))
                .icon(text_input::Icon {
                    font: icons::ICON_FONT,
                    code_point: icons::SEARCH,
                    size: None,
                    spacing: 4.0,
                    side: text_input::Side::Left,
                })
                .style(style::text_input);

            if size.width <= 300.0 {
                search_bar.into()
            } else {
                row![
                    container(container(search_bar).width(Fill).max_width(300)).align_left(Fill),
                    horizontal_space().width(Shrink),
                    top_right_row
                ]
                .spacing(4)
                .into()
            }
        } else {
            row![search_button, horizontal_space(), top_right_row]
                .spacing(4)
                .into()
        }
    });

    column![container(top_row).height(30), content,]
        .spacing(4.0)
        .into()
}
