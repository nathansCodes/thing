use std::{collections::HashMap, io};

use crate::{
    assets::{
        Asset, AssetHandle, AssetKind, AssetPath, AssetsData, AssetsMessage, Image, Mode,
        io::load_dir,
    },
    style,
    widgets::{self, dnd::dnd_provider, dropdown, icons},
};

use iced::{
    Alignment, Element,
    Length::{Fill, FillPortion, Shrink},
    Task, Theme,
    widget::{
        self, Rule, button, column, container, horizontal_rule, row, rule, scrollable, text,
        text_input,
    },
};
use iced_aw::ContextMenu;

use anyhow::{Result, anyhow};

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

            if !path.is_dir() {
                state.last_error = Some(
                    anyhow!(io::Error::from(io::ErrorKind::AlreadyExists))
                        .context(format!("Couldn't create {path:?}")),
                );
                Task::done(AssetsMessage::LoadFailed)
            } else {
                match load_dir(&path) {
                    Ok(files) => {
                        let (succeeded, failed) = {
                            #[allow(clippy::type_complexity)]
                            let (succeeded, failed): (
                                HashMap<u32, (AssetPath, Result<Asset>)>,
                                HashMap<u32, (AssetPath, Result<Asset>)>,
                            ) = files.into_iter().partition(|(_, (_, res))| res.is_ok());

                            let succeeded = succeeded
                                .into_iter()
                                .map(|(id, (asset_path, res))| (id, (asset_path, res.unwrap())))
                                .collect();

                            let failed = failed
                                .into_iter()
                                .map(|(id, (asset_path, res))| (id, asset_path, res.unwrap_err()));

                            (succeeded, failed)
                        };

                        let failed_messages = failed.into_iter().map(|(id, asset_path, err)| {
                            state.last_error = Some(
                                err.context(format!("Couldn't load asset {asset_path} ({id})")),
                            );

                            AssetsMessage::LoadAssetFailed(id, asset_path)
                        });

                        Task::batch(
                            std::iter::once(AssetsMessage::LoadCompleted(path, succeeded))
                                .chain(failed_messages)
                                .map(Task::done),
                        )
                    }
                    Err(err) => {
                        println!("{err}");
                        state.last_error =
                            Some(anyhow!(err).context(format!("Couldn't load {path:?}")));

                        Task::done(AssetsMessage::LoadFailed)
                    }
                }
            }
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
        AssetsMessage::EditAsset(handle) => Task::done(AssetsMessage::EditAsset(handle)),
        AssetsMessage::SetPayload(payload) => Task::done(AssetsMessage::SetPayload(payload)),
        AssetsMessage::QueryChanged(text) => {
            if !state.query_present() && text.is_some() {
                state.query = text;
                text_input::focus(state.search_bar.clone())
            } else {
                state.query = text;
                Task::none()
            }
        }
        AssetsMessage::ModeChanged(mode) => {
            state.mode = mode;
            Task::none()
        }
        AssetsMessage::ViewChanged(view) => {
            state.view = view;
            Task::none()
        }
        AssetsMessage::ShowHideModeDropdown => {
            state.mode_dropdown_open = !state.mode_dropdown_open;
            Task::none()
        }
        AssetsMessage::ShowHideViewDropdown => {
            state.view_dropdown_open = !state.view_dropdown_open;
            Task::none()
        }
        AssetsMessage::SetRenameInput(val) => {
            if state.rename_state.is_none() && val.is_some() {
                state.rename_state = val;
                text_input::focus(state.rename_input.clone())
            } else {
                state.rename_state = val;
                Task::none()
            }
        }
        AssetsMessage::RenameAsset => {
            let Some((handle, mut new_name)) = state.rename_state.take() else {
                return Task::none();
            };

            let Some(old_path) = state.index.get(&handle.0).cloned() else {
                return Task::none();
            };

            new_name = new_name.trim().to_string();

            if new_name.is_empty() {
                return Task::none();
            }

            let extension = old_path.extension();

            if new_name.ends_with(extension) {
                new_name.truncate(new_name.len() - extension.len());
            }

            let new_path = old_path.kind() + (new_name.clone() + extension);

            state.index.insert(handle.0, new_path.clone());

            let folder = state.folder.clone().unwrap();

            let from = folder.clone() + old_path.clone();

            let to = folder + new_path.clone();

            let err_ctx = format!("Couldn't rename {from:?} to {to:?}");

            if let Err(err) = std::fs::rename(from, to) {
                state.last_error = Some(anyhow!(err).context(err_ctx));

                return Task::done(AssetsMessage::RenameAssetFailed(handle));
            };

            match state.write_index() {
                Ok(_) => {
                    let asset = state.assets.remove(&old_path).unwrap();
                    state.assets.insert(new_path.clone(), asset);

                    Task::done(AssetsMessage::SetRenameInput(None))
                }
                Err(err) => {
                    state.last_error = Some(anyhow!(err).context(err_ctx));

                    Task::done(AssetsMessage::RenameAssetFailed(handle))
                }
            }
        }
        AssetsMessage::RenameAssetFailed(..) => Task::none(),
        AssetsMessage::LoadAssetFailed(..) => Task::none(),
        AssetsMessage::AddAssetToGraph(_) => Task::none(),
    }
}

pub fn image_item<'a>(
    i: usize,
    handle: AssetHandle,
    path: &'a AssetPath,
    state: &'a AssetsData,
    img: &'a Image,
) -> Element<'a, AssetsMessage> {
    let rename_input = state.rename_state.as_ref().and_then(|(rn_handle, input)| {
        (*rn_handle == handle).then_some(
            text_input("Rename...", input.as_str())
                .on_input(|input| AssetsMessage::SetRenameInput(Some((*rn_handle, input))))
                .on_submit(AssetsMessage::RenameAsset)
                .id(state.rename_input.clone()),
        )
    });

    let name = path.file_name();

    match state.mode {
        Mode::Thumbnails => button(
            column![
                widget::image(&img.handle)
                    .height(100.0)
                    .width(100.0)
                    .filter_method(widget::image::FilterMethod::Nearest),
                rename_input
                    .map(|ri| Element::from(ri.width(100.0).align_x(Alignment::Center)))
                    .unwrap_or(
                        text(name)
                            .width(100.0)
                            .center()
                            .size(15)
                            .wrapping(text::Wrapping::WordOrGlyph)
                            .into()
                    )
            ]
            .spacing(5.0)
            .padding(5.0),
        )
        .padding(0)
        .style(style::list_thumbnail)
        .on_press(AssetsMessage::OpenAsset(handle))
        .into(),
        Mode::List => button(
            row![
                widget::image(&img.handle)
                    .height(30)
                    .width(30)
                    .filter_method(widget::image::FilterMethod::Nearest),
                rename_input.map(Element::from).unwrap_or(text(name).into())
            ]
            .width(Fill)
            .height(40)
            .spacing(10)
            .padding(5)
            .align_y(Alignment::Center),
        )
        .on_press(AssetsMessage::OpenAsset(handle))
        .padding(0)
        .style(style::list_item(i.is_multiple_of(2)))
        .into(),
    }
}

pub fn view_controls(state: &AssetsData) -> Element<'_, AssetsMessage> {
    container({
        let mode_dropdown = dropdown(
            state.mode_dropdown_open,
            AssetsMessage::ShowHideModeDropdown,
            match state.mode {
                Mode::Thumbnails => icons::thumbnails(),
                Mode::List => icons::list(),
            },
            [
                (
                    icons::THUMBNAILS,
                    "Thumbnails",
                    AssetsMessage::ModeChanged(Mode::Thumbnails),
                ),
                (icons::LIST, "List", AssetsMessage::ModeChanged(Mode::List)),
            ]
            .into_iter(),
        );

        let view_dropdown = dropdown(
            state.view_dropdown_open,
            AssetsMessage::ShowHideViewDropdown,
            match state.view {
                AssetKind::Image => icons::image(),
                AssetKind::Character => icons::user(),
            },
            [
                (
                    icons::IMAGE,
                    "Images",
                    AssetsMessage::ViewChanged(AssetKind::Image),
                ),
                (
                    icons::USER,
                    "Characters",
                    AssetsMessage::ViewChanged(AssetKind::Character),
                ),
            ]
            .into_iter(),
        );

        let search_button = button(icons::search().center())
            .width(30)
            .on_press({
                if state.query.is_some() {
                    AssetsMessage::QueryChanged(None)
                } else {
                    AssetsMessage::QueryChanged(Some("".to_string()))
                }
            })
            .style(style::menu_button);

        let top_right_row = row![view_dropdown, mode_dropdown].spacing(4.0);

        row![search_button, top_right_row]
            .spacing(4)
            .align_y(Alignment::Center)
            .padding([0, 4])
            .width(Shrink)
            .height(Fill)
    })
    .width(Shrink)
    .into()
}

pub fn view(state: &AssetsData) -> Element<'_, AssetsMessage> {
    let mut images: Vec<_> = state
        .index
        .iter()
        .filter_map(|(id, asset_path)| {
            state.assets.get(asset_path).and_then(|asset| {
                let path_str = asset_path.to_string().to_lowercase();

                (path_str.starts_with(state.view.folder())
                    && path_str.contains(&state.query().to_lowercase()))
                .then_some(())
                .and_then(|_| match asset {
                    Asset::Image(img) => Some(img),
                    Asset::Character(character) => state.get_direct::<Image>(character.img),
                })
                .map(|img| (*id, asset_path, img))
            })
        })
        .collect();

    images.sort_by(|a, b| a.1.to_string().cmp(&b.1.to_string()));

    let images = images.into_iter().enumerate().map(|(i, (id, path, img))| {
        let handle = AssetHandle(id);
        let img_element = dnd_provider(
            AssetsMessage::SetPayload,
            crate::Draggable::Asset(handle),
            image_item(i, handle, path, state, img),
        );

        ContextMenu::new(img_element, move || {
            let extra_options = match path.kind() {
                AssetKind::Image => vec![],
                AssetKind::Character => vec![
                    widgets::menu_item_button("Add to Graph", None, None)
                        .on_press(AssetsMessage::AddAssetToGraph(AssetHandle(id)))
                        .width(Fill)
                        .into(),
                ],
            };

            container(
                column![
                    widgets::menu_item_button("Rename", None, Some(icons::RENAME),)
                        .on_press(AssetsMessage::SetRenameInput(Some((
                            handle,
                            path.name().to_string()
                        ))),)
                        .width(Fill),
                    widgets::menu_item_button("Edit", None, Some(icons::EDIT))
                        .on_press(AssetsMessage::EditAsset(handle))
                        .width(Fill)
                ]
                .push_maybe(
                    (!extra_options.is_empty()).then_some(horizontal_rule(8).style(
                        |theme: &Theme| rule::Style {
                            fill_mode: rule::FillMode::Padded(8),
                            ..rule::default(theme)
                        },
                    )),
                )
                .extend(extra_options),
            )
            .padding(4)
            .width(200)
            .style(style::dropdown)
            .into()
        })
        .into()
    });

    let search_bar = state.query.as_ref().map(|query| {
        column![
            text_input("Search...", query)
                .on_input(|input| AssetsMessage::QueryChanged(Some(input)))
                .icon(text_input::Icon {
                    font: icons::ICON_FONT,
                    code_point: icons::SEARCH,
                    size: None,
                    spacing: 4.0,
                    side: text_input::Side::Right,
                })
                .width(Fill)
                .padding([4, 6])
                .style(style::search_bar)
                .id(state.search_bar.clone()),
            horizontal_rule(1)
        ]
    });

    let layout = match state.mode {
        Mode::Thumbnails => Element::from(row(images).spacing(5).padding(3).width(Fill).wrap()),
        Mode::List => Element::from(column(images).spacing(2).width(Fill)),
    };

    let content = column![]
        .push_maybe(search_bar)
        .push(scrollable(layout).style(style::scrollable));

    content.into()
}
