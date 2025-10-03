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

impl AssetsData {
    pub fn update(&mut self, message: AssetsMessage) -> Task<AssetsMessage> {
        match message {
            AssetsMessage::LoadAssets(path) => match self.load(&path) {
                Ok(assets) => {
                    #[allow(clippy::type_complexity)]
                    let (succeeded, failed): (HashMap<_, _>, _) =
                        assets.into_iter().partition(|(_, (_, res))| res.is_ok());

                    let succeeded = succeeded
                        .into_iter()
                        .map(|(id, (asset_path, res))| (id, (asset_path, res.unwrap())));

                    let failed = failed
                        .into_iter()
                        .map(|(id, (asset_path, res))| (AssetHandle(id), res.unwrap_err()))
                        .collect::<Vec<_>>();

                    if failed.is_empty() {
                        Task::done(AssetsMessage::LoadCompleted(path, succeeded.collect()))
                    } else {
                        self.failed_loads = failed;

                        Task::done(AssetsMessage::LoadPartiallyFailed(
                            path,
                            succeeded.collect(),
                        ))
                    }
                }
                Err(err) => {
                    self.last_error = Some(err);

                    Task::done(AssetsMessage::LoadFailed)
                }
            },
            AssetsMessage::LoadCompleted(_, assets) => {
                for (id, (asset_path, asset)) in assets.into_iter() {
                    self.index.insert(id, asset_path.clone());
                    self.assets.insert(asset_path, asset);
                }

                Task::none()
            }
            AssetsMessage::LoadPartiallyFailed(path, succeeded) => {
                Task::done(AssetsMessage::LoadCompleted(path, succeeded))
            }
            AssetsMessage::LoadFailed => {
                if let Some(err) = &self.last_error {
                    println!("{err:#}");
                }

                Task::none()
            }
            AssetsMessage::OpenAsset(handle) => Task::done(AssetsMessage::OpenAsset(handle)),
            AssetsMessage::EditAsset(handle) => Task::done(AssetsMessage::EditAsset(handle)),
            AssetsMessage::SetPayload(payload) => Task::done(AssetsMessage::SetPayload(payload)),
            AssetsMessage::QueryChanged(text) => {
                if !self.query_present() && text.is_some() {
                    self.query = text;
                    text_input::focus(self.search_bar.clone())
                } else {
                    self.query = text;
                    Task::none()
                }
            }
            AssetsMessage::ModeChanged(mode) => {
                self.mode = mode;
                Task::none()
            }
            AssetsMessage::ViewChanged(view) => {
                self.view = view;
                Task::none()
            }
            AssetsMessage::ShowHideModeDropdown => {
                self.mode_dropdown_open = !self.mode_dropdown_open;
                Task::none()
            }
            AssetsMessage::ShowHideViewDropdown => {
                self.view_dropdown_open = !self.view_dropdown_open;
                Task::none()
            }
            AssetsMessage::SetRenameInput(val) => {
                if self.rename_state.is_none() && val.is_some() {
                    self.rename_state = val;
                    text_input::focus(self.rename_input.clone())
                } else {
                    self.rename_state = val;
                    Task::none()
                }
            }
            AssetsMessage::RenameAsset => {
                let Some((handle, new_name)) = self.rename_state.take() else {
                    return Task::none();
                };

                match self.rename(handle, new_name) {
                    Ok(_) => Task::none(),
                    Err(err) => {
                        self.last_error = Some(err);

                        Task::done(AssetsMessage::RenameAssetFailed(handle))
                    }
                }
            }
            AssetsMessage::RenameAssetFailed(..) => Task::none(),
            AssetsMessage::LoadAssetFailed(..) => Task::none(),
            AssetsMessage::AddAssetToGraph(_) => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, AssetsMessage> {
        let mut images: Vec<_> = self
            .index
            .iter()
            .filter_map(|(id, asset_path)| {
                self.assets.get(asset_path).and_then(|asset| {
                    let path_str = asset_path.to_string().to_lowercase();

                    (path_str.starts_with(self.view.folder())
                        && path_str.contains(&self.query().to_lowercase()))
                    .then_some(())
                    .and_then(|_| match asset {
                        Asset::Image(img) => Some(img),
                        Asset::Character(character) => self.get_direct::<Image>(character.img),
                    })
                    .map(|img| (*id, asset_path, img))
                })
            })
            .collect();

        images.sort_by(|a, b| a.1.to_string().cmp(&b.1.to_string()));

        let images =
            images.into_iter().enumerate().map(|(i, (id, path, img))| {
                let handle = AssetHandle(id);
                let img_element = dnd_provider(
                    AssetsMessage::SetPayload,
                    crate::Draggable::Asset(handle),
                    image_item(i, handle, path, self, img),
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
                        .push_maybe((!extra_options.is_empty()).then_some(
                            horizontal_rule(8).style(|theme: &Theme| rule::Style {
                                fill_mode: rule::FillMode::Padded(8),
                                ..rule::default(theme)
                            }),
                        ))
                        .extend(extra_options),
                    )
                    .padding(4)
                    .width(200)
                    .style(style::dropdown)
                    .into()
                })
                .into()
            });

        let search_bar = self.query.as_ref().map(|query| {
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
                    .id(self.search_bar.clone()),
                horizontal_rule(1)
            ]
        });

        let layout = match self.mode {
            Mode::Thumbnails => Element::from(row(images).spacing(5).padding(3).width(Fill).wrap()),
            Mode::List => Element::from(column(images).spacing(2).width(Fill)),
        };

        let content = column![]
            .push_maybe(search_bar)
            .push(scrollable(layout).style(style::scrollable));

        content.into()
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
