mod assets;
mod io;
mod notification;
mod style;
mod widgets;

use crate::notification::Notification;
use crate::widgets::dnd::{dnd_indicator, dnd_receiver};
use graph::connections::Edge;
use graph::line_styles::AxisAligned;
use graph::{GraphEvent, GraphNode, RelativeAttachment, line_styles};
use widgets::*;

use iced::keyboard::{Key, Modifiers};
use iced::widget::{horizontal_space, row, scrollable, slider, stack};
use iced::{
    Alignment, Border, Element, Font,
    Length::Fill,
    Padding, Point, Task, Theme,
    font::Weight,
    widget::{column, container, image, opaque, pane_grid, pane_grid::Configuration, text},
};
use iced::{Size, Subscription, Transformation, Vector, keyboard};
use iced_aw::{menu, menu::Item, menu_bar};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;

use crate::{
    assets::{AssetType, AssetsMessage, AssetsPane},
    graph::{Graph, GraphData},
    io::{IOError, pick_file, pick_folder},
};

fn main() -> iced::Result {
    iced::application("Hello", update, view)
        .subscription(subscription)
        .theme(|_| Theme::TokyoNight)
        .run_with(|| {
            (
                State {
                    graph_position: Vector::ZERO,
                    graph_zoom: 1.0,
                    folder: None,
                    nodes: GraphData::default(),
                    panes: pane_grid::State::with_configuration(Configuration::Split {
                        axis: pane_grid::Axis::Vertical,
                        ratio: 0.25,
                        a: Box::new(Configuration::Pane(Pane::Assets)),
                        b: Box::new(Configuration::Pane(Pane::Graph)),
                    }),
                    focus: None,
                    assets: AssetsPane::default(),
                    notifications: Vec::new(),
                    dnd_payload: None,
                },
                Task::none(),
            )
        })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Image {
    path: PathBuf,
}

#[derive(Clone, Debug)]
enum Draggable {
    ImageAsset(PathBuf),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Node {
    Character(Image),
    Family,
}

struct State {
    folder: Option<PathBuf>,
    graph_position: Vector,
    nodes: GraphData<Node, RelativeAttachment<line_styles::AxisAligned>>,
    panes: pane_grid::State<Pane>,
    assets: assets::AssetsPane,
    focus: Option<pane_grid::Pane>,
    notifications: Vec<Notification>,
    dnd_payload: Option<Draggable>,
    graph_zoom: f32,
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
    AddImage(PathBuf, Point),
    MenuButtonPressed,
    OpenAddImageDialog,
    OpenLoadFolderDialog,
    Save,
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),
    AssetsMessage(AssetsMessage),
    GraphEvent(GraphEvent<RelativeAttachment<line_styles::AxisAligned>>),
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
    TraverseGraph,
    SetDragPayload(Option<Draggable>),
    DropImageOnGraph(PathBuf, Point),
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
                let graph = Graph::new(&state.nodes, view_node)
                    .position(state.graph_position)
                    .zoom(state.graph_zoom)
                    .on_event(Message::GraphEvent)
                    .position_nodes(position_scheme)
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
                    row![
                        text(format!(
                            "Position: {} {}",
                            state.graph_position.x, state.graph_position.y
                        ))
                        .size(14.0),
                        horizontal_space(),
                        slider(0.5..=2.0, state.graph_zoom, |new_zoom| Message::GraphEvent(
                            GraphEvent::Zoom(new_zoom)
                        ))
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
                .padding(4.0)
                .width(Fill)
                .height(30.0)
                .style(style::info_bar);

                Element::from(dnd_receiver(
                    |payload, relative_cursor_pos| match payload {
                        Draggable::ImageAsset(path) => {
                            Some(Message::DropImageOnGraph(path, relative_cursor_pos))
                        }
                    },
                    state.dnd_payload.clone(),
                    column![
                        container(graph).padding(2.0).center_x(Fill).center_y(Fill),
                        info_bar,
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

    column![
        menu_bar,
        stack![
            dnd_indicator(
                state.dnd_payload.clone().map(|draggable| match draggable {
                    Draggable::ImageAsset(img_path) => container(
                        image(img_path.to_string_lossy().to_string())
                            .width(50.0)
                            .opacity(0.5)
                    )
                    .width(50.0)
                    .into(),
                }),
                container(grid)
            ),
            notifications
        ]
    ]
    .into()
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::AddImage(path, pos) => {
            if !path.exists() {
                Task::done(Message::AddImageFailed(IOError::IO(
                    std::io::ErrorKind::NotFound,
                )))
            } else {
                state
                    .nodes
                    .add(Node::Character(Image { path: path.clone() }), pos);

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
            Ok(path) => Message::AddImage(path, Point::ORIGIN),
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
                AssetType::Image => Task::done(Message::AddImage(path, Point::ORIGIN)),
            },
            AssetsMessage::LoadAssets(path) => {
                state.folder = Some(path.clone());
                assets::update(&mut state.assets, AssetsMessage::LoadAssets(path))
                    .map(Message::AssetsMessage)
            }
            AssetsMessage::LoadCompleted(_) | AssetsMessage::LoadFailed(_) => {
                assets::update(&mut state.assets, assets_message).map(Message::AssetsMessage)
            }
            AssetsMessage::SetPayload(payload) => Task::done(Message::SetDragPayload(payload)),
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
                    state.nodes = data;
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
        },
        Message::ImageButtonPressed => {
            println!("pressd");
            Task::none()
        }
        Message::MenuButtonPressed => Task::none(),
        Message::Save => {
            let parsed = toml::to_string(&state.nodes).unwrap();

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
        Message::TraverseGraph => {
            state
                .nodes
                .iter_bfs(0)
                .for_each(|(i, node)| println!("for_each {i}: {:?}", node.data()));
            Task::none()
        }
        Message::SetDragPayload(draggable) => {
            if !(state.dnd_payload.is_some() && draggable.is_some()) {
                state.dnd_payload = draggable;
            }
            Task::none()
        }
        Message::DropImageOnGraph(path, relative_cursor_pos) => {
            state.dnd_payload = None;
            Task::done(Message::AddImage(
                path,
                relative_cursor_pos * Transformation::scale(1.0 / state.graph_zoom)
                    - state.graph_position,
            ))
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
            _ => None,
        }),
        iced::time::every(Duration::from_millis(20)).map(|_| Message::Tick),
    ])
}

fn view_node(node: &Node) -> Element<'_, Message> {
    match node {
        Node::Character(img) => container(
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
    }
}

#[allow(clippy::type_complexity)]
fn position_scheme(
    prev: Option<(
        usize,
        &GraphNode<Node>,
        &RelativeAttachment<AxisAligned>,
        &RelativeAttachment<AxisAligned>,
        Size<f32>,
    )>,
    _id: usize,
    node: &GraphNode<Node>,
    size: iced::Size,
    data: &GraphData<Node, RelativeAttachment<line_styles::AxisAligned>>,
    layout: &iced::advanced::Layout<'_>,
    visited: Vec<usize>,
) -> Vector {
    let Some((prev_id, prev, attachment, prev_attachment, prev_size)) = prev else {
        return Vector::ZERO;
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
                                    if att.is_right() {
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

                    let leftmost = -children
                        .iter()
                        .fold(0.0, |acc, (_, size, _)| acc + size.width)
                        / 2.0
                        - (children.len() as f32 - 1.0) * padding;

                    let mut visited_children: Vec<_> = children
                        .iter()
                        .filter(|(child_id, _, _)| visited.contains(child_id))
                        .collect();

                    let num_visited_children = visited_children.len();

                    visited_children.extend(
                        children
                            .iter()
                            .enumerate()
                            .filter(|(i, (id, _, is_partner))| {
                                *i < num_visited_children && *is_partner && !visited.contains(id)
                            })
                            .map(|(_, a)| a),
                    );

                    let num_visited_children = visited_children.len();

                    let x = leftmost
                        + visited_children.iter().fold(0.0, |acc, (_, size, _)| {
                            acc + size.width
                                + padding
                                    * ((num_visited_children > 0) as i32 as f32 + num_partners)
                        });

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
                let x = match prev_attachment {
                    RelativeAttachment::Edge {
                        edge: Edge::Left, ..
                    } => -30.0,
                    RelativeAttachment::Edge {
                        edge: Edge::Right, ..
                    } => 20.0 + prev_size.width,
                    // TODO:
                    RelativeAttachment::Edge {
                        edge: Edge::Top, ..
                    } => todo!(),
                    _ => unreachable!(),
                };

                Vector::new(x, prev_size.height + 25.0)
            } else {
                unreachable!()
            }
        }
    }
}
