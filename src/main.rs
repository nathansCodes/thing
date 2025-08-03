mod assets;
mod graph;
mod io;
mod notification;
mod style;
mod widgets;

use crate::graph::{OnConnectEvent, RelativeAttachment, line_styles};
use crate::notification::Notification;
use crate::widgets::*;

use iced::keyboard::{Key, Modifiers};
use iced::widget::{horizontal_space, row, scrollable, stack};
use iced::{
    Alignment, Border, Element, Font,
    Length::Fill,
    Padding, Point, Task, Theme,
    font::Weight,
    widget::{column, container, image, opaque, pane_grid, pane_grid::Configuration, text},
};
use iced::{Subscription, keyboard};
use iced_aw::{menu, menu::Item, menu_bar};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;

use crate::{
    assets::{AssetType, AssetsMessage, AssetsPane},
    graph::{Graph, GraphData, NodeDraggedEvent},
    io::{IOError, pick_file, pick_folder},
};

fn main() -> iced::Result {
    iced::application("Hello", update, view)
        .subscription(subscription)
        .theme(|_| Theme::TokyoNight)
        .run_with(|| {
            (
                State {
                    folder: None,
                    images: GraphData::default(),
                    panes: pane_grid::State::with_configuration(Configuration::Split {
                        axis: pane_grid::Axis::Vertical,
                        ratio: 0.25,
                        a: Box::new(Configuration::Pane(Pane::Assets)),
                        b: Box::new(Configuration::Pane(Pane::Graph)),
                    }),
                    focus: None,
                    assets: AssetsPane::default(),
                    notifications: Vec::new(),
                },
                Task::none(),
            )
        })
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Image {
    path: PathBuf,
}

struct State {
    folder: Option<PathBuf>,
    images: GraphData<Image, RelativeAttachment<line_styles::AxisAligned>>,
    panes: pane_grid::State<Pane>,
    assets: assets::AssetsPane,
    focus: Option<pane_grid::Pane>,
    notifications: Vec<Notification>,
}

#[derive(Default)]
enum Pane {
    #[default]
    Assets,
    Graph,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddImage(PathBuf),
    MenuButtonPressed,
    OpenAddImageDialog,
    OpenLoadFolderDialog,
    Save,
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    AssetsMessage(AssetsMessage),
    NodeDragged(NodeDraggedEvent),
    NodeConnect(OnConnectEvent<RelativeAttachment<line_styles::AxisAligned>>),
    NodeDisconnect(usize),
    NodeDeleted(usize),
    ImageButtonPressed,
    Saved,
    SaveFailed(std::io::ErrorKind),
    DataLoaded(String),
    LoadData(PathBuf),
    LoadDataFailed(IOError),
    OpenFolderFailed(IOError),
    AddImageFailed(IOError),
    DismissNotification(usize),
    DataNotLoaded,
    Tick,
}

fn view(state: &State) -> Element<'_, Message> {
    #[rustfmt::skip]
    let menu_bar = menu_bar![
        (menu_button("File"), menu!(
            (menu_item_button("Open Folder", Some("CTRL+O")).on_press(Message::OpenLoadFolderDialog))
            (menu_item_button("Add Image", None).on_press(Message::OpenAddImageDialog))
            (menu_item_button("Save", Some("CTRL+S")).on_press(Message::Save))
        ).width(200.0).spacing(2.0))
    ].width(Fill).style(style::menu_bar).padding([2.5, 5.0]).spacing(5.0);

    let grid = pane_grid(&state.panes, |id, pane, _| {
        let is_focused = state.focus == Some(id);

        let mut title_bar_font = Font::DEFAULT;
        title_bar_font.weight = Weight::Bold;

        let title_bar = pane_grid::TitleBar::new(
            container(
                text(match pane {
                    Pane::Assets => "Assets",
                    Pane::Graph => "Graph",
                })
                .size(14.0)
                .font(title_bar_font),
            )
            .padding(Padding::new(2.0).left(8.0).right(8.0))
            .align_y(Alignment::Center),
        )
        .style(if is_focused {
            style::title_bar_focused
        } else {
            style::title_bar_active
        });

        let mut content = pane_grid::Content::new(match pane {
            Pane::Graph => {
                let graph = Graph::new(&state.images, view_image)
                    .on_node_dragged(Message::NodeDragged)
                    .node_attachments(RelativeAttachment::<line_styles::AxisAligned>::all_edges())
                    .on_connect(Message::NodeConnect)
                    .on_disconnect(Message::NodeDisconnect)
                    .on_delete(Message::NodeDeleted)
                    .allow_self_connections(true)
                    .allow_similar_connections(true);

                let graph = container(graph).style(|theme: &Theme| {
                    container::Style::default()
                        .background(theme.extended_palette().background.base.color)
                });

                Element::from(
                    container(Element::from(graph))
                        .center_x(Fill)
                        .center_y(Fill),
                )
            }
            Pane::Assets => container(assets::view(&state.assets).map(Message::AssetsMessage))
                .padding(Padding::new(5.0))
                .into(),
        })
        .style(match pane {
            Pane::Graph => |_: &Theme| container::Style::default(),
            _ => {
                if is_focused {
                    style::pane_focused
                } else {
                    style::pane_active
                }
            }
        });

        if !matches!(pane, Pane::Graph) {
            content = content.title_bar(title_bar);
        }

        content
    })
    .width(Fill)
    .height(Fill)
    .spacing(5.0)
    .on_click(Message::PaneClicked)
    .on_drag(Message::PaneDragged)
    .on_resize(10, Message::PaneResized);

    let notifications = row![
        horizontal_space(),
        opaque(
            scrollable(
                column(
                    state
                        .notifications
                        .iter()
                        .enumerate()
                        .map(|(i, notification)| widgets::notification(i, notification).into())
                )
                .spacing(10.0)
                .padding(5.0)
            )
            .height(500.0)
        )
    ];

    column![menu_bar, stack![container(grid), notifications]].into()
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::AddImage(path) => {
            if !path.exists() {
                Task::done(Message::AddImageFailed(IOError::IO(
                    std::io::ErrorKind::NotFound,
                )))
            } else {
                state
                    .images
                    .add(Image { path: path.clone() }, Point::ORIGIN);

                Task::none()
            }
        }
        Message::AddImageFailed(err) => {
            state.notifications.push(Notification::error(
                "Failed to add image",
                format!("Failed to add image: {err:?}",),
            ));

            Task::none()
        }
        Message::PaneClicked(pane) => {
            state.focus = Some(pane);
            Task::none()
        }
        Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
            state.panes.resize(split, ratio);
            Task::none()
        }
        Message::PaneDragged(pane_grid::DragEvent::Dropped { pane, target }) => {
            state.panes.drop(pane, target);
            Task::none()
        }
        Message::PaneDragged(_) => Task::none(),
        Message::OpenAddImageDialog => Task::perform(pick_file(), |path_maybe| match path_maybe {
            Ok(path) => Message::AddImage(path),
            Err(err) => Message::AddImageFailed(err),
        }),
        Message::OpenLoadFolderDialog => {
            Task::perform(pick_folder(), |path_maybe| match path_maybe {
                Ok(path) if path.exists() && path.is_dir() => Message::LoadData(path),
                Ok(path) if path.is_file() => {
                    Message::OpenFolderFailed(IOError::IO(std::io::ErrorKind::NotADirectory))
                }
                Ok(_) => Message::OpenFolderFailed(IOError::IO(std::io::ErrorKind::NotFound)),
                Err(err) => Message::OpenFolderFailed(err),
            })
        }
        Message::AssetsMessage(assets_message) => match assets_message {
            AssetsMessage::OpenAsset(asset_type, path) => match asset_type {
                AssetType::Image => Task::done(Message::AddImage(path)),
            },
            AssetsMessage::LoadAssets(path) => {
                state.folder = Some(path.clone());
                assets::update(&mut state.assets, AssetsMessage::LoadAssets(path))
                    .map(Message::AssetsMessage)
            }
            AssetsMessage::LoadCompleted(_) | AssetsMessage::LoadFailed(_) => {
                assets::update(&mut state.assets, assets_message).map(Message::AssetsMessage)
            }
        },
        Message::LoadData(path) => Task::batch([
            Task::done(Message::AssetsMessage(AssetsMessage::LoadAssets(
                path.clone(),
            ))),
            Task::perform(io::load(path), |res| match res {
                Ok(raw_data) => Message::DataLoaded(raw_data),
                Err(IOError::IO(ErrorKind::NotFound)) => Message::DataNotLoaded,
                Err(err) => Message::LoadDataFailed(err),
            }),
        ]),
        Message::LoadDataFailed(err) => {
            state.notifications.push(Notification::error(
                "Failed to load data",
                format!("Failed to load data: {err:?}",),
            ));
            Task::none()
        }
        Message::DataNotLoaded => Task::none(),
        Message::DataLoaded(raw_data) => {
            let data = toml::from_str(&raw_data).map_err(IOError::from);

            match data {
                Ok(data) => {
                    state.images = data;
                    Task::none()
                }
                Err(err) => Task::done(Message::LoadDataFailed(err)),
            }
        }
        Message::OpenFolderFailed(err) => {
            state.notifications.push(Notification::error(
                "Failed to open folder",
                format!("Failed to open folder: {err:?}",),
            ));
            Task::none()
        }
        Message::NodeDragged(event) => {
            if let Ok(img) = state.images.get_mut(event.id) {
                img.move_to(event.new_position);
            }
            Task::none()
        }
        Message::NodeConnect(OnConnectEvent {
            a,
            a_attachment,
            b,
            b_attachment,
        }) => {
            let _ = state.images.connect(a, a_attachment, b, b_attachment);
            Task::none()
        }
        Message::NodeDisconnect(i) => {
            state.images.disconnect(i);
            Task::none()
        }
        Message::NodeDeleted(i) => {
            state.images.remove(i);
            Task::none()
        }
        Message::ImageButtonPressed => {
            println!("pressd");
            Task::none()
        }
        Message::MenuButtonPressed => Task::none(),
        Message::Save => {
            let parsed = toml::to_string(&state.images).unwrap();

            if let Some(folder) = &state.folder {
                Task::perform(
                    io::save(folder.clone().join("data.toml"), parsed),
                    |res| match res {
                        Ok(()) => Message::Saved,
                        Err(err) => Message::SaveFailed(err.kind()),
                    },
                )
            } else {
                Task::none()
            }
        }
        Message::Saved => {
            state.notifications.push(Notification::info(
                "Saved successfully!",
                format!(
                    "successfully saved to {}",
                    state.folder.as_ref().unwrap().to_string_lossy()
                ),
            ));
            Task::none()
        }
        Message::SaveFailed(err) => {
            state.notifications.push(Notification::error(
                "Failed to save data",
                format!(
                    "Failed to save to {}: {err}",
                    state.folder.as_ref().unwrap().to_string_lossy()
                ),
            ));
            Task::none()
        }
        Message::DismissNotification(i) => {
            state.notifications.remove(i);

            Task::none()
        }
        Message::Tick => {
            if let Some(first_notification) = state.notifications.get_mut(0) {
                first_notification.timeout -= 0.02;

                if first_notification.timeout <= 0.0 {
                    state.notifications.remove(0);
                }
            }

            Task::none()
        }
    }
}

fn subscription(_state: &State) -> Subscription<Message> {
    Subscription::batch([
        keyboard::on_key_press(|key, modifiers| match (modifiers, key) {
            (Modifiers::CTRL, Key::Character(char)) if char.eq("s") => Some(Message::Save),
            (Modifiers::CTRL, Key::Character(char)) if char.eq("o") => {
                Some(Message::OpenLoadFolderDialog)
            }
            _ => None,
        }),
        iced::time::every(Duration::from_millis(20)).map(|_| Message::Tick),
    ])
}

fn view_image(img: &Image) -> Element<'_, Message> {
    container(
        column![
            image(img.path.clone())
                .width(Fill)
                .height(Fill)
                .filter_method(image::FilterMethod::Nearest),
            opaque(
                column![
                    text(img.path.file_name().unwrap().to_str().unwrap().to_owned())
                        .center()
                        .width(Fill),
                    base_button("ahkdlfjs").on_press(Message::ImageButtonPressed)
                ]
                .width(Fill)
                .spacing(5.0)
            ),
        ]
        .spacing(5.0),
    )
    .width(150.0)
    .height(150.0)
    .padding(5.0)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        container::Style::default()
            .background(palette.background.base.color)
            .border(
                Border::default()
                    .rounded(15.0)
                    .width(2.0)
                    .color(palette.background.weak.color),
            )
    })
    .into()
}
