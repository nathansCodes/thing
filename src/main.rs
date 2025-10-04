mod assets_pane;
mod inspector;
mod notification;
mod positioning_schemes;
mod style;
mod widgets;

use crate::assets_pane::{AssetsPane, pick_file, pick_folder};
use crate::inspector::{Inspector, InspectorMessage};
use crate::notification::Notification;
use crate::widgets::dialog::{Dialog, DialogOption};
use crate::widgets::dnd::{dnd_indicator, dnd_receiver};
use asset_system::AssetsData;
use asset_system::image::DEFAULT_IMAGE;
use asset_system::io::AssetsError;
use asset_system::{Asset, AssetHandle, AssetKind, Character, io};
use assets_pane::AssetsMessage;
use graph::connections::Edge;
use graph::{GraphEvent, RelativeAttachment, line_styles};
use iced::Length::Shrink;
use iced::keyboard::key::Named;
use ron::ser::PrettyConfig;
use widgets::*;

use anyhow::anyhow;
use iced::keyboard::{Key, Modifiers};
use iced::widget::{
    horizontal_rule, horizontal_space, row, rule, scrollable, slider, stack, vertical_space,
};
use iced::{Alignment, Settings, Size, Subscription, Transformation, Vector, keyboard, window};
use iced::{
    Element, Font,
    Length::Fill,
    Padding, Point, Task, Theme,
    font::Weight,
    widget::{column, container, image, opaque, pane_grid, pane_grid::Configuration, text},
};
use iced_aw::{menu, menu::Item, menu_bar};
use serde::{Deserialize, Serialize};

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use crate::graph::GraphData;

fn main() -> iced::Result {
    iced::application("Hello", update, view)
        .window(window::Settings {
            min_size: Some(Size::new(650.0, 500.0)),
            ..Default::default()
        })
        .settings(Settings {
            fonts: vec![include_bytes!("../fonts/things.ttf").as_slice().into()],
            ..Default::default()
        })
        .subscription(subscription)
        .theme(|_| Theme::TokyoNight)
        .transparent(true)
        .run_with(|| {
            (
                State {
                    nodes: GraphData::default(),
                    assets: AssetsData::default(),
                    assets_pane: AssetsPane::default(),
                    panes: pane_grid::State::with_configuration(Configuration::Split {
                        axis: pane_grid::Axis::Vertical,
                        ratio: 0.25,
                        a: Box::new(Configuration::Pane(Pane::Assets)),
                        b: Box::new(Configuration::Split {
                            axis: pane_grid::Axis::Vertical,
                            ratio: 0.7,
                            a: Box::new(Configuration::Pane(Pane::Graph)),
                            b: Box::new(Configuration::Pane(Pane::Inspector)),
                        }),
                    }),
                    focus: None,
                    graph_position: Vector::ZERO,
                    graph_zoom: 1.0,
                    notifications: Vec::new(),
                    dnd_payload: None,
                    dialog: None,
                    last_error: None,
                    inspector: Inspector::default(),
                },
                Task::none(),
            )
        })
}

#[derive(Clone, Debug)]
enum Draggable {
    Asset(AssetHandle),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Node {
    Character(AssetHandle),
    Family,
}

struct State {
    nodes: GraphData<Node, RelativeAttachment<line_styles::AxisAligned>>,
    assets: asset_system::AssetsData,
    assets_pane: assets_pane::AssetsPane,
    panes: pane_grid::State<Pane>,
    focus: Option<pane_grid::Pane>,
    graph_position: Vector,
    graph_zoom: f32,
    notifications: Vec<Notification>,
    dnd_payload: Option<Draggable>,
    dialog: Option<Dialog<Message>>,
    last_error: Option<anyhow::Error>,
    inspector: Inspector,
}

enum Pane {
    Assets,
    Graph,
    Inspector,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
enum Message {
    AssetsMessage(AssetsMessage),
    AddCharacter(AssetHandle, Point),
    MenuButtonPressed,
    OpenLoadFolderDialog,
    ConfirmLoadPath(PathBuf),
    Load(PathBuf),
    LoadData(PathBuf),
    ParseData(String, PathBuf),
    LoadDataFailed,
    OpenFolderFailed,
    OpenAddAssetDialog,
    AddExternalAsset(PathBuf),
    AddAsset(String, Asset),
    AddAssetFailed,
    Save,
    Saved,
    SaveFailed,
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    GraphEvent(GraphEvent<RelativeAttachment<line_styles::AxisAligned>>),
    TraverseGraph,
    DismissNotification(usize),
    Tick,
    SetDragPayload(Option<Draggable>),
    DropAssetOnGraph(AssetHandle, Point),
    CloseDialog,
    EscapePressed,
    NotifyError,
    InspectorMessage(InspectorMessage),
    Undo,
    Redo,
}

impl From<AssetsMessage> for Message {
    fn from(value: AssetsMessage) -> Self {
        Self::AssetsMessage(value)
    }
}

impl From<InspectorMessage> for Message {
    fn from(value: InspectorMessage) -> Self {
        Self::InspectorMessage(value)
    }
}

impl From<GraphEvent<RelativeAttachment<line_styles::AxisAligned>>> for Message {
    fn from(value: GraphEvent<RelativeAttachment<line_styles::AxisAligned>>) -> Self {
        Self::GraphEvent(value)
    }
}

fn view(state: &State) -> Element<'_, Message> {
    #[rustfmt::skip]
    let menu_bar = menu_bar![
        (
            menu_button("File", Message::MenuButtonPressed, None),
            menu!(
                (menu_item_button("Open Folder", Some("CTRL+O"), None).on_press(Message::OpenLoadFolderDialog))
                (menu_item_button("Add Image", None, None).on_press(Message::OpenAddAssetDialog))
                (menu_item_button("Save", Some("CTRL+S"), None).on_press(Message::Save))
            )
            .width(200.0)
            .spacing(2.0)
        )
        (
            menu_button("Graph", Message::MenuButtonPressed, None),
            menu!(
                (menu_item_button("Select All", Some("CTRL+A"), None).on_press(Message::GraphEvent(GraphEvent::SelectAll)))
            )
            .width(200.0)
            .spacing(2.0)
        )
    ].width(Fill).style(style::menu_bar).padding([2.5, 5.0]).spacing(5.0);

    let grid = pane_grid(&state.panes, |id, pane, _| {
        let is_focused = state.focus == Some(id);

        let mut title_bar_font = Font::DEFAULT;
        title_bar_font.weight = Weight::Semibold;

        let title = container(
            container(
                text(match pane {
                    Pane::Assets => "Assets",
                    Pane::Graph => "Graph",
                    Pane::Inspector => "Inspector",
                })
                .size(15.0)
                .font(title_bar_font),
            )
            .padding(Padding::new(4.0).left(8).right(8))
            .style(if is_focused {
                style::title_bar_label_focused
            } else {
                style::title_bar_label
            }),
        )
        .style(style::title_bar_label_container);

        let controls_full = container(match pane {
            Pane::Assets => Element::from(
                state
                    .assets_pane
                    .view_controls()
                    .map(Message::AssetsMessage),
            ),
            Pane::Graph => "".into(),
            Pane::Inspector => Element::from(
                state
                    .inspector
                    .view_controls()
                    .map(Message::InspectorMessage),
            ),
        })
        .height(40);

        let controls_compact = row![].height(40);

        let title_bar = pane_grid::TitleBar::new(title)
            .controls(pane_grid::Controls::dynamic(
                controls_full,
                controls_compact,
            ))
            .style(if is_focused {
                style::title_bar_focused
            } else {
                style::title_bar
            })
            .always_show_controls()
            .padding(2);

        let content = match pane {
            Pane::Graph => view_graph(state),
            Pane::Assets => container(
                state
                    .assets_pane
                    .view(&state.assets)
                    .map(Message::AssetsMessage),
            )
            .padding(2)
            .into(),
            Pane::Inspector => state.inspector.view(state).map(Message::InspectorMessage),
        };

        let r1 = horizontal_rule(1).style(move |theme: &Theme| rule::Style {
            color: if is_focused {
                theme.extended_palette().primary.base.color
            } else {
                rule::default(theme).color
            },
            width: 2,
            fill_mode: rule::FillMode::Padded(2),
            ..rule::default(theme)
        });

        let r2 = horizontal_rule(1).style(move |theme: &Theme| rule::Style {
            color: if is_focused {
                theme.extended_palette().primary.weak.color
            } else {
                theme.extended_palette().background.weak.color
            },
            width: 1,
            fill_mode: rule::FillMode::Padded(2),
            ..rule::default(theme)
        });

        let mut content = pane_grid::Content::new(if !matches!(pane, Pane::Graph) {
            Element::from(column![r1, r2, content])
        } else {
            content
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
        container(opaque(
            scrollable(
                column(
                    state
                        .notifications
                        .iter()
                        .enumerate()
                        .map(|(i, notification)| widgets::notification(i, notification).into())
                )
                .height(Shrink)
                .spacing(10.0)
                .padding(5.0)
            )
            .style(style::scrollable)
        ))
        .align_right(Fill)
        .max_height(500.0)
    ];

    let payload = state.dnd_payload.clone().map(|draggable| {
        let img = match draggable {
            Draggable::Asset(handle) => state.assets.get(handle).map(|asset| match asset {
                Asset::Image(img) => img.handle.clone(),
                Asset::Character(character) => state
                    .assets
                    .get_direct::<asset_system::Image>(character.img)
                    .unwrap_or(&DEFAULT_IMAGE)
                    .handle
                    .clone(),
            }),
        };

        container(
            image(img.as_ref().unwrap_or(&DEFAULT_IMAGE.handle))
                .width(50.0)
                .opacity(0.5),
        )
        .width(50.0)
        .into()
    });

    widgets::dialog(
        &state.dialog,
        column![
            menu_bar,
            stack![
                dnd_indicator(payload, container(grid).padding(2)),
                notifications
            ]
        ],
    )
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::AssetsMessage(assets_message) => match assets_message {
            AssetsMessage::OpenAsset(handle) => state
                .inspector
                .update(
                    &mut state.assets,
                    InspectorMessage::SetCurrent(handle, false),
                )
                .map(Message::InspectorMessage),
            AssetsMessage::AddAssetToGraph(handle) => {
                let Some(asset) = state.assets.get(handle) else {
                    return Task::none();
                };
                match asset {
                    Asset::Image(_) => {
                        let name = state
                            .assets
                            .path(handle)
                            .unwrap()
                            .file_name()
                            .split('.')
                            .next()
                            .unwrap()
                            .to_string();

                        let result = state
                            .assets
                            .add(name.clone() + ".chara.ron", Character { name, img: handle });

                        match result {
                            Ok(handle) => Task::done(Message::AddCharacter(
                                handle,
                                Point::ORIGIN + state.graph_position,
                            )),
                            Err(err) => {
                                state.last_error = Some(err);

                                Task::done(Message::NotifyError)
                            }
                        }
                    }
                    Asset::Character(_) => Task::done(Message::AddCharacter(
                        handle,
                        Point::ORIGIN + state.graph_position,
                    )),
                }
            }
            AssetsMessage::LoadAssets(path) => {
                state.assets.set_folder(path.clone());
                state
                    .assets_pane
                    .update(&mut state.assets, AssetsMessage::LoadAssets(path.clone()))
                    .map(Message::AssetsMessage)
            }
            AssetsMessage::LoadCompleted(path) => state
                .assets_pane
                .update(
                    &mut state.assets,
                    AssetsMessage::LoadCompleted(path.clone()),
                )
                .map(Message::AssetsMessage)
                .chain(Task::done(Message::LoadData(path))),
            AssetsMessage::LoadPartiallyFailed(path) => {
                for (path, err) in state.assets.failed_loads() {
                    state.notifications.push(Notification::error(
                        "Failed to load asset.",
                        format!("Couldn't load {path}: {err}"),
                    ));
                }

                state
                    .assets_pane
                    .update(&mut state.assets, AssetsMessage::LoadPartiallyFailed(path))
                    .map(Message::AssetsMessage)
            }
            AssetsMessage::SetPayload(payload) => Task::done(Message::SetDragPayload(payload)),
            AssetsMessage::RenameAssetFailed(_) => {
                if let Some(err) = state.assets_pane.last_error() {
                    state.notifications.push(Notification::error(
                        "Failed to rename asset",
                        format!("{err:#}"),
                    ));
                }

                Task::none()
            }
            AssetsMessage::LoadAssetFailed(..) => {
                if let Some(err) = state.assets_pane.last_error() {
                    state.notifications.push(Notification::error(
                        "Failed to load asset",
                        format!("{err:#}"),
                    ));
                }

                Task::none()
            }
            AssetsMessage::EditAsset(handle) => Task::batch([
                Task::done(Message::from(InspectorMessage::SetCurrent(handle, true))),
                Task::done(Message::from(InspectorMessage::EnterEditMode)),
            ]),
            _ => state
                .assets_pane
                .update(&mut state.assets, assets_message)
                .map(Message::AssetsMessage),
        },
        Message::AddCharacter(chara, pos) => {
            state.nodes.add(Node::Character(chara), pos);
            Task::none()
        }
        Message::MenuButtonPressed => Task::none(),
        Message::OpenLoadFolderDialog => match pick_folder() {
            Ok(path) if path.exists() && path.is_dir() => {
                Task::done(Message::ConfirmLoadPath(path))
            }
            Ok(path) if path.is_file() => {
                state.last_error = Some(anyhow!(std::io::Error::from(
                    std::io::ErrorKind::NotADirectory
                )));
                Task::done(Message::OpenFolderFailed)
            }
            Ok(_) => {
                state.last_error =
                    Some(anyhow!(std::io::Error::from(std::io::ErrorKind::NotFound)));
                Task::done(Message::OpenFolderFailed)
            }
            Err(err) => {
                state.last_error = Some(err);
                Task::done(Message::OpenFolderFailed)
            }
        },
        Message::ConfirmLoadPath(path) => {
            let read_dir: Vec<_> = path.as_path().read_dir().unwrap().collect();

            if read_dir.iter().any(|path| {
                path.as_ref()
                    .is_ok_and(|entry| entry.file_name() == "data.ron")
            }) {
                return Task::done(Message::Load(path));
            } else if read_dir.len()
                - read_dir
                    .iter()
                    .filter(|entry| {
                        entry.as_ref().is_ok_and(|entry| {
                            entry.path().is_dir()
                                && AssetKind::from_str(
                                    entry.file_name().as_os_str().to_str().unwrap(),
                                )
                                .is_ok()
                        })
                    })
                    .count()
                > 0
            {
                state.dialog = Some(Dialog::new(
                    "Are you sure you want to load this folder?",
                    format!(
                        "Directory {path:?} already contains files. Are you sure you want to use this folder?"
                    ),
                    Message::CloseDialog,
                    vec![DialogOption::new(
                        dialog::Severity::Warn,
                        "Load Anyway",
                        Message::Load(path),
                    )],
                ))
            }

            Task::none()
        }
        Message::Load(path) => {
            state.dialog = None;

            Task::done(Message::AssetsMessage(AssetsMessage::LoadAssets(path)))
        }
        Message::LoadData(path) => Task::done(match io::load(&path) {
            Ok(raw_data) => Message::ParseData(raw_data, path.clone()),
            Err(err) => {
                state.last_error = Some(err);

                Message::LoadDataFailed
            }
        }),
        Message::ParseData(raw_data, path) => {
            let data = ron::from_str(&raw_data);

            match data {
                Ok(data) => {
                    println!("{data:#?}");
                    state.nodes = data;

                    state.notifications.push(Notification::info(
                        "Successfully loaded data!",
                        format!("Successfully loaded data from {:?}", path.clone()),
                    ));

                    state.assets.set_folder(path.clone());

                    Task::none()
                }
                Err(err) => {
                    println!("{err}");
                    state.last_error = Some(anyhow!(err));
                    Task::done(Message::LoadDataFailed)
                }
            }
        }
        Message::LoadDataFailed => {
            if let Some(err) = &state.last_error
                && let Some(assets_err) = err.downcast_ref::<AssetsError>()
                && *assets_err != AssetsError::DialogClosed
            {
                state.notifications.push(Notification::error(
                    "Failed to load data",
                    format!("Failed to load data: {err}",),
                ));
            }

            Task::none()
        }
        Message::OpenFolderFailed => {
            let Some(err) = &state.last_error else {
                return Task::none();
            };

            if let Some(AssetsError::DialogClosed) = err.downcast_ref::<AssetsError>() {
                return Task::none();
            }

            state.notifications.push(Notification::error(
                "Failed to open folder",
                format!("Failed to open folder: {err:?}",),
            ));

            Task::none()
        }
        Message::OpenAddAssetDialog => match pick_file() {
            Ok(path) => Task::done(Message::AddExternalAsset(path)),
            Err(err) => {
                state.last_error = Some(err);
                Task::done(Message::AddAssetFailed)
            }
        },
        Message::AddExternalAsset(path) => {
            let res = state.assets.copy_to_assets_dir(&path);

            Task::done(match res {
                Ok((path, asset)) => Message::AddAsset(
                    path.file_name().unwrap().to_string_lossy().to_string(),
                    asset,
                ),
                Err(err) => {
                    state.last_error = Some(err);
                    Message::AddAssetFailed
                }
            })
        }
        Message::AddAsset(file_name, asset) => match state.assets.add(file_name, asset) {
            Ok(_) => Task::none(),
            Err(err) => {
                state.last_error = Some(anyhow!(err));
                Task::done(Message::AddAssetFailed)
            }
        },
        Message::AddAssetFailed => {
            if let Some(err) = &state.last_error {
                state.notifications.push(Notification::error(
                    "Failed to add asset",
                    format!("Failed to add asset: {err:?}",),
                ));
            }

            Task::none()
        }
        Message::Save => {
            let Some(folder) = state.assets.folder() else {
                return Task::none();
            };

            match state.assets.write_assets() {
                Ok(failed) => {
                    if !failed.is_empty() {
                        for (_, err) in failed {
                            state.notifications.push(Notification::error(
                                "Failed to save asset.",
                                err.to_string(),
                            ));
                        }

                        state.last_error = None;
                        return Task::done(Message::SaveFailed);
                    }

                    let parsed =
                        ron::ser::to_string_pretty(&state.nodes, PrettyConfig::new()).unwrap();

                    match io::save(folder, parsed) {
                        Ok(()) => Task::done(Message::Saved),
                        Err(err) => {
                            state.last_error = Some(err);
                            Task::done(Message::SaveFailed)
                        }
                    }
                }
                Err(err) => {
                    state.last_error = Some(err);

                    Task::done(Message::SaveFailed)
                }
            }
        }
        Message::Saved => {
            state.notifications.push(Notification::info(
                "Saved successfully!",
                format!(
                    "successfully saved to {}",
                    state.assets.folder().unwrap().to_string_lossy()
                ),
            ));
            Task::none()
        }
        Message::SaveFailed => {
            state.notifications.push(Notification::error(
                "Failed to save data",
                format!(
                    "Failed to save to {}: {}",
                    state.assets.folder().unwrap().to_string_lossy(),
                    state
                        .last_error
                        .as_ref()
                        .map_or(String::new(), |err| err.to_string())
                ),
            ));

            Task::none()
        }
        Message::PaneClicked(pane) => {
            state.focus = Some(pane);
            Task::none()
        }
        Message::PaneDragged(pane_grid::DragEvent::Dropped { pane, target }) => {
            state.panes.drop(pane, target);
            Task::none()
        }
        Message::PaneDragged(_) => Task::none(),
        Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
            state.panes.resize(split, ratio);
            Task::none()
        }
        Message::GraphEvent(ev) => match ev {
            GraphEvent::Move(new_position) => {
                state.graph_position = new_position - Point::ORIGIN;
                Task::none()
            }
            GraphEvent::Zoom(zoom) => {
                state.graph_zoom = zoom;
                Task::none()
            }
            GraphEvent::MoveNode {
                id,
                new_position,
                was_dragged: _,
            } => {
                if let Some(img) = state.nodes.get_mut(id) {
                    img.move_to(new_position);
                }
                Task::none()
            }
            GraphEvent::Connect {
                a,
                a_attachment,
                b,
                b_attachment,
            } => {
                let a_node = state.nodes.get(a).unwrap().data();
                let b_node = state.nodes.get(b).unwrap().data();

                if (matches!(a_node, Node::Family) && a_attachment.is_horizontal())
                    || (matches!(b_node, Node::Family) && b_attachment.is_horizontal())
                {
                    return Task::none();
                }

                let Ok(a_edge) = Edge::try_from(a_attachment.clone()) else {
                    return Task::none();
                };

                let Ok(b_edge) = Edge::try_from(b_attachment.clone()) else {
                    return Task::none();
                };

                let family_present =
                    matches!(a_node, Node::Family) || matches!(b_node, Node::Family);

                if !family_present {
                    let a_point = state.nodes.get(a).unwrap().position();
                    let b_point = state.nodes.get(b).unwrap().position();

                    let halfway_point = a_point + (b_point - a_point) * 0.5;

                    match (a_edge, b_edge) {
                        (Edge::Left, Edge::Right) => {
                            let _ = state.nodes.attach_new(
                                Node::Family,
                                halfway_point,
                                RelativeAttachment::top(),
                                a,
                                RelativeAttachment::left(),
                            );

                            let _ = state.nodes.connect(
                                b,
                                b_attachment,
                                state.nodes.num_nodes() - 1,
                                RelativeAttachment::top(),
                            );
                        }
                        (Edge::Right, Edge::Left) => {
                            let _ = state.nodes.attach_new(
                                Node::Family,
                                halfway_point,
                                RelativeAttachment::top(),
                                a,
                                RelativeAttachment::right(),
                            );

                            let _ = state.nodes.connect(
                                b,
                                b_attachment,
                                state.nodes.num_nodes() - 1,
                                RelativeAttachment::top(),
                            );
                        }
                        _ => (),
                    }

                    return Task::none();
                }

                let _ = state.nodes.connect(a, a_attachment, b, b_attachment);

                Task::none()
            }
            GraphEvent::Disconnect { connection_id } => {
                state.nodes.remove_connection(connection_id);
                Task::none()
            }
            GraphEvent::Delete { id } => {
                state.nodes.remove(id);
                Task::none()
            }
            GraphEvent::ConnectionDropped { id, attachment } => {
                println!("Connection dropped: {id}-{attachment:?}");
                Task::none()
            }
            GraphEvent::Select(id) => {
                state.nodes.select(id);
                Task::none()
            }
            GraphEvent::Deselect(id) => {
                state.nodes.deselect(id);
                Task::none()
            }
            GraphEvent::ClearSelection => {
                state.nodes.clear_selection();
                Task::none()
            }
            GraphEvent::SelectAll => {
                state.nodes.select_all();
                Task::none()
            }
        },
        Message::TraverseGraph => {
            state
                .nodes
                .iter_bfs(state.nodes.selection().next().unwrap_or(0))
                .for_each(|(i, node)| println!("for_each {i}: {:?}", node.data()));
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
        Message::SetDragPayload(draggable) => {
            if !(state.dnd_payload.is_some() && draggable.is_some()) {
                state.dnd_payload = draggable;
            }
            Task::none()
        }
        Message::DropAssetOnGraph(handle, relative_cursor_pos) => {
            println!("dropping {handle:?}");
            state.dnd_payload = None;
            match &state.assets[handle] {
                Asset::Image(_) => Task::none(),
                Asset::Character(_) => Task::done(Message::AddCharacter(
                    handle,
                    relative_cursor_pos * Transformation::scale(1.0 / state.graph_zoom)
                        + state.graph_position,
                )),
            }
        }
        Message::CloseDialog => {
            state.dialog = None;
            Task::none()
        }
        Message::EscapePressed => {
            if state.dialog.is_some() {
                Task::done(Message::CloseDialog)
            } else if state.assets_pane.is_renaming() {
                Task::done(Message::AssetsMessage(AssetsMessage::SetRenameInput(None)))
            } else if state.assets_pane.query_present() {
                Task::done(Message::AssetsMessage(AssetsMessage::QueryChanged(None)))
            } else {
                Task::none()
            }
        }
        Message::NotifyError => {
            if let Some(err) = &state.last_error {
                state.notifications.push(Notification::error(
                    "Failed to add character",
                    format!("{err:#}"),
                ));
            }

            Task::none()
        }
        Message::InspectorMessage(edit_message) => state
            .inspector
            .update(&mut state.assets, edit_message)
            .map(Message::InspectorMessage),
        Message::Undo => match state.focus.and_then(|id| state.panes.get(id)) {
            None | Some(Pane::Graph) => Task::none(),
            Some(Pane::Inspector) => state
                .inspector
                .update(&mut state.assets, InspectorMessage::Undo)
                .map(Message::InspectorMessage),
            Some(Pane::Assets) => Task::none(),
        },
        Message::Redo => match state.focus.and_then(|id| state.panes.get(id)) {
            None | Some(Pane::Graph) => Task::none(),
            Some(Pane::Inspector) => state
                .inspector
                .update(&mut state.assets, InspectorMessage::Redo)
                .map(Message::InspectorMessage),
            Some(Pane::Assets) => Task::none(),
        },
    }
}

fn subscription(_state: &State) -> Subscription<Message> {
    Subscription::batch([
        keyboard::on_key_press(|key, modifiers| match (modifiers, key) {
            (Modifiers::CTRL, Key::Character(char)) if char.eq("s") => Some(Message::Save),
            (Modifiers::CTRL, Key::Character(char)) if char.eq("o") => {
                Some(Message::OpenLoadFolderDialog)
            }
            (Modifiers::CTRL, Key::Character(char)) if char.eq("t") => Some(Message::TraverseGraph),
            (Modifiers::CTRL, Key::Character(char)) if char.eq("a") => {
                Some(Message::GraphEvent(GraphEvent::SelectAll))
            }
            (Modifiers::CTRL, Key::Character(char)) if char.eq("f") => Some(
                Message::AssetsMessage(AssetsMessage::QueryChanged(Some("".to_string()))),
            ),
            (Modifiers::CTRL, Key::Character(char)) if char.eq("z") => Some(Message::Undo),
            (Modifiers::CTRL, Key::Character(char)) if char.eq("y") => Some(Message::Redo),
            (_, Key::Named(Named::Escape)) if modifiers.is_empty() => Some(Message::EscapePressed),
            _ => None,
        }),
        iced::time::every(Duration::from_millis(20)).map(|_| Message::Tick),
    ])
}

fn view_graph(state: &State) -> Element<'_, Message> {
    let graph = graph(&state.nodes, node(&state.assets))
        .position(state.graph_position)
        .zoom(state.graph_zoom)
        .on_event(Message::GraphEvent)
        .position_nodes(positioning_schemes::family_tree)
        .per_node_attachments(|node| {
            match node {
                Node::Character(_) => vec![
                    (RelativeAttachment::top(), Vector::new(0.15, 0.15)),
                    (RelativeAttachment::left(), Vector::new(0.15, 0.15)),
                    (RelativeAttachment::right(), Vector::new(0.15, 0.15)),
                ],
                Node::Family => vec![
                    (RelativeAttachment::top(), Vector::new(1.0, 1.0)),
                    (RelativeAttachment::bottom(), Vector::new(1.0, 1.0)),
                ],
            }
            .into_iter()
        })
        .allow_self_connections(true)
        .allow_similar_connections(true);

    let mut zoom_text = (state.graph_zoom * 100.0).round().to_string();
    zoom_text.retain(|c| c != '.');

    let info_bar = container(
        container(
            row![
                text(format!(
                    "Position: {} {}",
                    state.graph_position.x, state.graph_position.y
                ))
                .size(14.0),
                horizontal_space(),
                slider(0.5..=2.0, state.graph_zoom, |new_zoom| {
                    Message::GraphEvent(GraphEvent::Zoom(new_zoom))
                })
                .width(100.0)
                .step(0.05)
                .style(style::info_bar_zoom_slider),
                text(zoom_text + "%")
                    .size(13.0)
                    .width(35.0)
                    .align_x(Alignment::End),
            ]
            .spacing(4.0)
            .height(Fill)
            .align_y(Alignment::Center),
        )
        .padding([4.0, 8.0])
        .width(Fill)
        .height(30.0)
        .style(style::info_bar),
    )
    .style(style::info_bar_border)
    .padding(Padding::new(2.0).bottom(1.0));

    Element::from(dnd_receiver(
        |payload, relative_cursor_pos| match payload {
            Draggable::Asset(handle) => state
                .assets
                .is::<asset_system::Character>(handle)
                .then_some(Message::DropAssetOnGraph(handle, relative_cursor_pos)),
        },
        state.dnd_payload.clone(),
        stack![
            container(graph)
                .padding(2.0)
                .center_x(Fill)
                .center_y(Fill)
                .style(container::transparent),
            column![vertical_space(), info_bar,].padding(4.0)
        ],
    ))
}
