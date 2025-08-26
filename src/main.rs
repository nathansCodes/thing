mod assets;
mod io;
mod notification;
mod style;
mod widgets;

use crate::assets::{Asset, AssetHandle, Image};
use crate::notification::Notification;
use crate::widgets::dialog::{Dialog, DialogOption};
use crate::widgets::dnd::{dnd_indicator, dnd_receiver};
use graph::connections::Edge;
use graph::line_styles::AxisAligned;
use graph::{GraphEvent, GraphNode, RelativeAttachment, line_styles};
use iced::Length::Shrink;
use iced::keyboard::key::Named;
use ron::ser::PrettyConfig;
use widgets::*;

use iced::keyboard::{Key, Modifiers};
use iced::widget::{horizontal_space, row, scrollable, slider, stack, vertical_space};
use iced::{
    Alignment, Border, Element, Font,
    Length::Fill,
    Padding, Point, Task, Theme,
    font::Weight,
    widget::{column, container, image, opaque, pane_grid, pane_grid::Configuration, text},
};
use iced::{Rectangle, Settings, Size, Subscription, Transformation, Vector, keyboard, window};
use iced_aw::{menu, menu::Item, menu_bar};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;

use crate::{
    assets::{AssetsData, AssetsMessage},
    graph::GraphData,
    io::{IOError, pick_file, pick_folder},
};

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
        .run_with(|| {
            (
                State {
                    nodes: GraphData::default(),
                    assets: AssetsData::default(),
                    panes: pane_grid::State::with_configuration(Configuration::Split {
                        axis: pane_grid::Axis::Vertical,
                        ratio: 0.25,
                        a: Box::new(Configuration::Pane(Pane::Assets)),
                        b: Box::new(Configuration::Pane(Pane::Graph)),
                    }),
                    focus: None,
                    graph_position: Vector::ZERO,
                    graph_zoom: 1.0,
                    notifications: Vec::new(),
                    dnd_payload: None,
                    dialog: None,
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
struct Character {
    name: String,
    img: AssetHandle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Node {
    Character(Character),
    Family,
}

struct State {
    nodes: GraphData<Node, RelativeAttachment<line_styles::AxisAligned>>,
    assets: assets::AssetsData,
    panes: pane_grid::State<Pane>,
    focus: Option<pane_grid::Pane>,
    graph_position: Vector,
    graph_zoom: f32,
    notifications: Vec<Notification>,
    dnd_payload: Option<Draggable>,
    dialog: Option<Dialog<Message>>,
}

#[derive(Default, PartialEq)]
enum Pane {
    #[default]
    Assets,
    Graph,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
enum Message {
    AssetsMessage(AssetsMessage),
    AddCharacter(Character, Point),
    MenuButtonPressed,
    OpenLoadFolderDialog,
    ConfirmLoadPath(PathBuf),
    DataNotLoaded,
    LoadData(PathBuf),
    ParseData(String, PathBuf),
    LoadDataFailed(IOError),
    OpenFolderFailed(IOError),
    OpenAddAssetDialog,
    AddExternalAsset(PathBuf),
    AddAsset(String, Asset),
    AddAssetFailed(IOError),
    Save,
    Saved,
    SaveFailed(std::io::ErrorKind),
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    GraphEvent(GraphEvent<RelativeAttachment<line_styles::AxisAligned>>),
    TraverseGraph,
    CharacterButtonPressed,
    DismissNotification(usize),
    Tick,
    SetDragPayload(Option<Draggable>),
    DropAssetOnGraph(AssetHandle, Point),
    CloseDialog,
}

fn view(state: &State) -> Element<'_, Message> {
    #[rustfmt::skip]
    let menu_bar = menu_bar![
        (
            menu_button("File", Message::MenuButtonPressed),
            menu!(
                (menu_item_button("Open Folder", Some("CTRL+O")).on_press(Message::OpenLoadFolderDialog))
                (menu_item_button("Add Image", None).on_press(Message::OpenAddAssetDialog))
                (menu_item_button("Save", Some("CTRL+S")).on_press(Message::Save))
            )
            .width(200.0)
            .spacing(2.0)
        )
        (
            menu_button("Graph", Message::MenuButtonPressed),
            menu!(
                (menu_item_button("Select All", Some("CTRL+A")).on_press(Message::GraphEvent(GraphEvent::SelectAll)))
            )
            .width(200.0)
            .spacing(2.0)
        )
    ].width(Fill).style(style::menu_bar).padding([2.5, 5.0]).spacing(5.0);

    let grid = pane_grid(&state.panes, |id, pane, _| {
        let is_focused = state.focus == Some(id);

        let mut title_bar_font = Font::DEFAULT;
        title_bar_font.weight = Weight::Semibold;

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
                let graph = graph(&state.nodes, view_node(&state.assets))
                    .position(state.graph_position)
                    .zoom(state.graph_zoom)
                    .on_event(Message::GraphEvent)
                    .position_nodes(family_tree)
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
                        Draggable::Asset(handle) => match state.assets[handle] {
                            Asset::Image(_) => {
                                Some(Message::DropAssetOnGraph(handle, relative_cursor_pos))
                            }
                        },
                    },
                    state.dnd_payload.clone(),
                    stack![
                        container(graph).padding(2.0).center_x(Fill).center_y(Fill),
                        column![vertical_space(), info_bar,].padding(4.0)
                    ],
                ))
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

        if *pane != Pane::Graph {
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
            container(
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
            )
            .max_height(500.0)
        )
    ];

    widgets::dialog(
        &state.dialog,
        column![
            menu_bar,
            stack![
                dnd_indicator(
                    state.dnd_payload.clone().map(|draggable| match draggable {
                        Draggable::Asset(handle) => match &state.assets[handle] {
                            Asset::Image(img) =>
                                container(image(img.handle.clone()).width(50.0).opacity(0.5))
                                    .width(50.0)
                                    .into(),
                        },
                    }),
                    container(grid)
                ),
                notifications
            ]
        ],
    )
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::AssetsMessage(assets_message) => match assets_message {
            AssetsMessage::OpenAsset(handle) => {
                let Some(asset) = state.assets.get(handle) else {
                    return Task::none();
                };
                match asset {
                    Asset::Image(img) => Task::done(Message::AddCharacter(
                        Character {
                            name: img.file_name.split('.').next().unwrap().to_string(),
                            img: handle,
                        },
                        Point::ORIGIN + state.graph_position,
                    )),
                }
            }
            AssetsMessage::LoadAssets(path) => {
                state.assets.set_folder(path.clone());
                assets::update(&mut state.assets, AssetsMessage::LoadAssets(path.clone()))
                    .map(Message::AssetsMessage)
                    .chain(Task::perform(
                        io::load(path.clone()),
                        move |res| match res {
                            Ok(raw_data) => Message::ParseData(raw_data, path.clone()),
                            Err(IOError::IO(ErrorKind::NotFound)) => Message::DataNotLoaded,
                            Err(err) => Message::LoadDataFailed(err),
                        },
                    ))
            }
            AssetsMessage::SetPayload(payload) => Task::done(Message::SetDragPayload(payload)),
            _ => assets::update(&mut state.assets, assets_message).map(Message::AssetsMessage),
        },
        Message::AddCharacter(chara, pos) => {
            state.nodes.add(Node::Character(chara), pos);
            Task::none()
        }
        Message::MenuButtonPressed => Task::none(),
        Message::OpenLoadFolderDialog => {
            Task::perform(pick_folder(), |path_maybe| match path_maybe {
                Ok(path) if path.exists() && path.is_dir() => Message::ConfirmLoadPath(path),
                Ok(path) if path.is_file() => {
                    Message::OpenFolderFailed(IOError::IO(std::io::ErrorKind::NotADirectory))
                }
                Ok(_) => Message::OpenFolderFailed(IOError::IO(std::io::ErrorKind::NotFound)),
                Err(err) => Message::OpenFolderFailed(err),
            })
        }
        Message::ConfirmLoadPath(path) => {
            let read_dir: Vec<_> = path.as_path().read_dir().unwrap().collect();

            if read_dir.iter().any(|path| {
                path.as_ref()
                    .is_ok_and(|entry| entry.file_name() == "data.ron")
            }) {
                return Task::done(Message::LoadData(path));
            } else if read_dir.len()
                - read_dir
                    .iter()
                    .filter(|entry| {
                        entry.as_ref().is_ok_and(|entry| {
                            entry.path().is_dir() && entry.file_name() == "images"
                        })
                    })
                    .count()
                > 0
            {
                state.dialog = Some(Dialog::new(
                    "Are you sure you want to load this folder?",
                    format!(
                        "Directory {path:?} already contains files. Are you sure you want to use it this folder?"
                    ),
                    Message::CloseDialog,
                    vec![DialogOption::new(
                        dialog::Severity::Warn,
                        "Load Anyway",
                        Message::LoadData(path),
                    )],
                ))
            }

            Task::none()
        }
        Message::DataNotLoaded => Task::none(),
        Message::LoadData(path) => {
            state.dialog = None;

            Task::done(Message::AssetsMessage(AssetsMessage::LoadAssets(path)))
        }
        Message::ParseData(raw_data, path) => {
            let data: Result<
                GraphData<Node, RelativeAttachment<line_styles::AxisAligned>>,
                IOError,
            > = ron::from_str(&raw_data).map_err(IOError::from);

            match data {
                Ok(data) => {
                    state.nodes = data;

                    state.notifications.push(Notification::info(
                        "Successfully loaded data!",
                        format!("Successfully loaded data from {:?}", path.clone()),
                    ));

                    state.assets.set_folder(path.clone());

                    Task::none()
                }
                Err(err) => Task::done(Message::LoadDataFailed(err)),
            }
        }
        Message::LoadDataFailed(err) => {
            state.notifications.push(Notification::error(
                "Failed to load data",
                format!("Failed to load data: {err}",),
            ));

            Task::none()
        }
        Message::OpenFolderFailed(err) => {
            if let IOError::DialogClosed = err {
                return Task::none();
            }

            state.notifications.push(Notification::error(
                "Failed to open folder",
                format!("Failed to open folder: {err:?}",),
            ));

            Task::none()
        }
        Message::OpenAddAssetDialog => Task::perform(pick_file(), |path_maybe| match path_maybe {
            Ok(path) => Message::AddExternalAsset(path),
            Err(err) => Message::AddAssetFailed(err),
        }),
        Message::AddExternalAsset(path) => {
            let closure = move |res: Result<(PathBuf, Asset), IOError>| match res {
                Ok((path, asset)) => Message::AddAsset(
                    path.file_name().unwrap().to_string_lossy().to_string(),
                    asset,
                ),
                Err(err) => Message::AddAssetFailed(err),
            };

            Task::perform(
                io::copy_to_assets_dir(state.assets.folder().cloned(), path.clone()),
                closure,
            )
        }
        Message::AddAsset(file_name, asset) => match state.assets.add(file_name, asset) {
            Ok(_) => Task::none(),
            Err(err) => Task::done(Message::AddAssetFailed(err)),
        },
        Message::AddAssetFailed(err) => {
            state.notifications.push(Notification::error(
                "Failed to add asset",
                format!("Failed to add asset: {err:?}",),
            ));

            Task::none()
        }
        Message::Save => {
            let parsed = ron::ser::to_string_pretty(&state.nodes, PrettyConfig::new()).unwrap();

            if let Some(folder) = &state.assets.folder() {
                Task::perform(io::save(folder.join("data.ron"), parsed), |res| match res {
                    Ok(()) => Message::Saved,
                    Err(err) => Message::SaveFailed(err.kind()),
                })
            } else {
                Task::none()
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
        Message::SaveFailed(err) => {
            state.notifications.push(Notification::error(
                "Failed to save data",
                format!(
                    "Failed to save to {}: {err}",
                    state.assets.folder().unwrap().to_string_lossy()
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
        Message::CharacterButtonPressed => {
            println!("pressd");
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
            state.dnd_payload = None;
            match &state.assets[handle] {
                Asset::Image(image) => Task::done(Message::AddCharacter(
                    Character {
                        name: image
                            .file_name
                            .split('.')
                            .next()
                            .unwrap_or("name")
                            .to_string(),
                        img: handle,
                    },
                    relative_cursor_pos * Transformation::scale(1.0 / state.graph_zoom)
                        + state.graph_position,
                )),
            }
        }
        Message::CloseDialog => {
            state.dialog = None;
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
            (Modifiers::CTRL, Key::Character(char)) if char.eq("t") => Some(Message::TraverseGraph),
            (Modifiers::CTRL, Key::Character(char)) if char.eq("a") => {
                Some(Message::GraphEvent(GraphEvent::SelectAll))
            }
            (_, Key::Named(Named::Escape)) if modifiers.is_empty() => Some(Message::CloseDialog),
            _ => None,
        }),
        iced::time::every(Duration::from_millis(20)).map(|_| Message::Tick),
    ])
}

#[allow(clippy::type_complexity)]
fn view_node<'a>(
    assets: &'a AssetsData,
) -> Box<dyn for<'any> Fn(&'any GraphNode<Node>) -> Element<'any, Message> + 'a> {
    Box::new(|node| match node.data() {
        Node::Character(chara) => container(
            column![
                image(Image::try_from(&assets[chara.img]).unwrap().handle)
                    .width(Fill)
                    .height(Fill)
                    .filter_method(image::FilterMethod::Nearest),
                opaque(
                    column![
                        text(&chara.name).center().width(Fill),
                        base_button("ahkdlfjs").on_press(Message::CharacterButtonPressed)
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
        .style(style::node(node.selected()))
        .into(),
        Node::Family => container("")
            .width(10.0)
            .height(10.0)
            .style(|theme: &Theme| container::Style {
                text_color: None,
                background: Some(theme.palette().success.into()),
                border: Border::default().rounded(10.0),
                ..Default::default()
            })
            .into(),
    })
}

#[allow(clippy::type_complexity)]
fn family_tree(
    prev: Option<(
        usize,
        &GraphNode<Node>,
        &RelativeAttachment<AxisAligned>,
        &RelativeAttachment<AxisAligned>,
        Size<f32>,
    )>,
    id: usize,
    node: &GraphNode<Node>,
    size: iced::Size,
    data: &GraphData<Node, RelativeAttachment<line_styles::AxisAligned>>,
    layout: &iced::advanced::Layout<'_>,
    visited: Vec<usize>,
) -> Vector {
    let Some((prev_id, prev, attachment, prev_attachment, prev_size)) = prev else {
        let total_covered_space = visited
            .iter()
            .filter_map(|other_id| {
                (other_id != &id).then_some(()).and_then(|_| {
                    layout
                        .children()
                        .nth(*other_id)
                        .map(|layout| layout.bounds())
                })
            })
            .fold(
                Rectangle::new(layout.position(), Size::ZERO),
                |acc, bounds| acc.union(&bounds),
            );

        return Vector::new(total_covered_space.width + 75.0, 0.0);
    };

    match node.data() {
        Node::Character(_) => match prev.data() {
            Node::Character(_) => unreachable!(),
            Node::Family => match prev_attachment {
                RelativeAttachment::Edge {
                    edge: Edge::Bottom, ..
                } => {
                    let mut children: Vec<_> = data
                        .get_connections(prev_id)
                        .filter_map(|conn| {
                            matches!(
                                conn.0,
                                RelativeAttachment::Edge {
                                    edge: Edge::Bottom,
                                    ..
                                }
                            )
                            .then_some((
                                conn.1,
                                layout.children().nth(conn.1).unwrap().bounds().size(),
                                None,
                            ))
                        })
                        .collect();

                    let num_visited_partners =
                        add_partners_to_children(&mut children, data, layout, &visited) as f32;

                    let padding = 25.0;

                    let leftmost = -(children
                        .iter()
                        .fold(0.0, |acc, (_, size, _)| acc + size.width)
                        + (children.len() as f32 - 1.0) * padding)
                        / 2.0;

                    let visited_children: Vec<_> = children
                        .iter()
                        .filter(|(child_id, _, partner_edge)| {
                            visited.contains(child_id)
                                || partner_edge.as_ref().is_some_and(|partners_partner| {
                                    visited.contains(partners_partner)
                                })
                        })
                        .collect();

                    let x = leftmost
                        + visited_children
                            .iter()
                            .fold(0.0, |acc, (_, size, _)| acc + size.width + padding)
                        + padding * num_visited_partners;

                    Vector::new(x, 75.0)
                }
                RelativeAttachment::Edge {
                    edge: Edge::Top, ..
                } => {
                    let x = match attachment {
                        RelativeAttachment::Edge {
                            edge: Edge::Left, ..
                        } => 30.0,
                        RelativeAttachment::Edge {
                            edge: Edge::Right, ..
                        } => -20.0 - size.width,
                        _ => unreachable!(),
                    };

                    Vector::new(x, -25.0 - size.height)
                }
                _ => unreachable!(),
            },
        },
        Node::Family => {
            if let Node::Character(_) = prev.data() {
                match prev_attachment {
                    RelativeAttachment::Edge {
                        edge: Edge::Left, ..
                    } => Vector::new(-30.0, prev_size.height + 25.0),
                    RelativeAttachment::Edge {
                        edge: Edge::Right, ..
                    } => Vector::new(20.0 + prev_size.width, prev_size.height + 25.0),
                    RelativeAttachment::Edge {
                        edge: Edge::Top, ..
                    } => {
                        let mut children: Vec<_> = data
                            .get_connections(id)
                            .filter_map(|conn| {
                                matches!(
                                    conn.0,
                                    RelativeAttachment::Edge {
                                        edge: Edge::Bottom,
                                        ..
                                    }
                                )
                                .then_some((
                                    conn.1,
                                    layout.children().nth(conn.1).unwrap().bounds().size(),
                                    false,
                                ))
                            })
                            .collect();

                        let mut num_partners = 0.0;

                        for (i, (child_id, _, _)) in children.clone().iter().enumerate() {
                            for (att, other_id, _) in data.get_connections(*child_id) {
                                if let Node::Family = data.get(other_id).unwrap().data()
                                    && att.is_horizontal()
                                {
                                    let Some(partner_id) = data.get_connections(other_id).find_map(
                                        |(family_att, partner_id, _)| {
                                            (family_att.is_top() && other_id != *child_id)
                                                .then_some(partner_id)
                                        },
                                    ) else {
                                        continue;
                                    };

                                    let partner_size =
                                        layout.children().nth(partner_id).unwrap().bounds().size();

                                    let elem = (partner_id, partner_size, true);

                                    if !children.contains(&elem) {
                                        if att.is_right() && i < children.len() {
                                            children.insert(i, elem);
                                            num_partners += 1.0;
                                        } else if att.is_left() {
                                            children.insert(i + 1, elem);
                                            num_partners += 1.0;
                                        }
                                    }
                                }
                            }
                        }

                        let padding = 25.0;

                        let children_width = children
                            .iter()
                            .fold(0.0, |acc, (_, size, _)| acc + size.width)
                            + (children.len() as f32 + num_partners) * padding;

                        Vector::new(children_width / 2.0, -75.0)
                    }
                    _ => unreachable!(),
                }
            } else {
                unreachable!()
            }
        }
    }
}

fn add_partners_to_children<S: graph::line_styles::LineStyle + PartialEq + Send>(
    children: &mut Vec<(usize, Size, Option<usize>)>,
    data: &GraphData<Node, RelativeAttachment<S>>,
    layout: &iced::advanced::Layout<'_>,
    visited: &[usize],
) -> usize {
    let mut num_visited_partners = 0;

    for (i, (child_id, _, _)) in children.clone().iter().enumerate() {
        for (att, other_id, _) in data.get_connections(*child_id) {
            if let Node::Family = data.get(other_id).unwrap().data()
                && att.is_horizontal()
            {
                let Some(partner_id) =
                    data.get_connections(other_id)
                        .find_map(|(family_att, partner_id, _)| {
                            (family_att.is_top() && other_id != *child_id).then_some(partner_id)
                        })
                else {
                    continue;
                };

                let partner_size = layout.children().nth(partner_id).unwrap().bounds().size();

                if !children.iter().any(|(id, _, _)| *id == partner_id) {
                    if att.is_right() && i < children.len() {
                        children.insert(i + 1, (partner_id, partner_size, Some(*child_id)));
                        if visited.contains(child_id) {
                            num_visited_partners += 1;
                        }
                    } else if att.is_left() {
                        children.insert(i, (partner_id, partner_size, Some(*child_id)));
                        if visited.contains(child_id) {
                            num_visited_partners += 1;
                        }
                    }
                }
            }
        }
    }

    num_visited_partners
}
