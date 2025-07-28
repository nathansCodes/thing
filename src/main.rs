mod assets;
mod file_picker;
mod graph;
mod style;

use crate::graph::{Attachment, OnConnectEvent};

use iced::{
    Alignment, Border, Element, Font,
    Length::Fill,
    Padding, Point, Task, Theme, Vector,
    font::Weight,
    widget::{
        canvas::Path, column, container, image, pane_grid, pane_grid::Configuration, pick_list,
        row, text,
    },
};
use std::path::PathBuf;

use crate::{
    assets::{AssetType, AssetsMessage, AssetsPane},
    file_picker::{IOError, pick_file, pick_folder},
    graph::{Graph, GraphData, NodeDraggedEvent},
};

fn main() -> iced::Result {
    iced::application("Hello", update, view)
        .theme(|_| Theme::CatppuccinMocha)
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
                },
                Task::none(),
            )
        })
}

#[derive(Clone, Debug, PartialEq)]
pub enum Family {
    Parent,
    Child,
    SiblingLeft,
    SiblingRight,
}

impl Attachment for Family {
    fn connection_point(&self) -> iced::Vector {
        match self {
            Family::Parent => Vector::new(0.5, 0.0),
            Family::Child => Vector::new(0.5, 1.0),
            Family::SiblingLeft => Vector::new(0.0, 0.5),
            Family::SiblingRight => Vector::new(1.0, 0.5),
        }
    }

    fn path(a: Self, a_point: Point, b: Self, other_point: Point) -> iced::widget::canvas::Path {
        let mut a_point = a_point;
        let mut b_point = other_point;

        // swap if needed
        let (a, b) = match (a, b) {
            (Self::SiblingLeft, Self::SiblingRight) => {
                std::mem::swap(&mut a_point, &mut b_point);

                (Self::SiblingRight, Self::SiblingLeft)
            }
            (Self::Child, Self::Parent) => {
                std::mem::swap(&mut a_point, &mut b_point);

                (Self::Parent, Self::Child)
            }
            (a, b)
                if (a == Self::Parent || a == Self::Child)
                    && (b == Self::SiblingLeft || b == Self::SiblingRight) =>
            {
                std::mem::swap(&mut a_point, &mut b_point);

                (b, a.clone())
            }
            others => others,
        };

        let a_direction = match a {
            Self::Parent => Vector::new(0.0, -1.0),
            Self::Child => Vector::new(0.0, 1.0),
            Self::SiblingLeft => Vector::new(-1.0, 0.0),
            Self::SiblingRight => Vector::new(1.0, 0.0),
        };

        let b_direction = match b {
            Self::Parent => Vector::new(0.0, -1.0),
            Self::Child => Vector::new(0.0, 1.0),
            Self::SiblingLeft => Vector::new(-1.0, 0.0),
            Self::SiblingRight => Vector::new(1.0, 0.0),
        };

        Path::new(|builder| {
            builder.move_to(a_point);

            let a_vector = Vector::new(a_point.x, a_point.y);
            let b_vector = Vector::new(b_point.x, b_point.y);

            let a_stub = a_point + a_direction * 25.0;
            let b_stub = b_point + b_direction * 25.0;

            match (a, b) {
                // if they're the same
                (a, b) if a == b => {
                    let (connecting_start, connecting_end) = match a_direction {
                        Vector { x: 0.0, y } => {
                            let mut connecting_start = a_stub;
                            let mut connecting_end = b_stub;

                            let y = if y == 1.0 {
                                f32::max(connecting_start.y, connecting_end.y)
                            } else {
                                f32::min(connecting_start.y, connecting_end.y)
                            };

                            connecting_start.y = y;
                            connecting_end.y = y;

                            (connecting_start, connecting_end)
                        }
                        Vector { x, y: 0.0 } => {
                            let mut connecting_start = a_stub;
                            let mut connecting_end = b_stub;

                            let x = if x == 1.0 {
                                f32::max(connecting_start.x, connecting_end.x)
                            } else {
                                f32::min(connecting_start.x, connecting_end.x)
                            };

                            connecting_start.x = x;
                            connecting_end.x = x;

                            (connecting_start, connecting_end)
                        }
                        _ => unreachable!(),
                    };

                    builder.line_to(connecting_start);
                    builder.line_to(connecting_end);

                    builder.line_to(b_point);
                }
                // side and other side
                // or top and bottom
                (Self::Parent, Self::Child) | (Self::SiblingRight, Self::SiblingLeft) => {
                    let halfway_vector = (b_vector - a_vector) * 0.5;

                    // builder.bezier_curve_to(
                    //     Point::new(
                    //         a_point.x - halfway_vector.x * a_direction.x,
                    //         a_point.y - halfway_vector.y * a_direction.y,
                    //     ),
                    //     Point::new(
                    //         b_point.x - halfway_vector.x * b_direction.x,
                    //         b_point.y - halfway_vector.y * b_direction.y,
                    //     ),
                    //     b_point,
                    // );
                    builder.line_to(a_stub);

                    // TODO: if a > b, then fix

                    if a_stub.x * a_direction.x.abs() < b_stub.x * b_direction.x.abs()
                        || a_stub.y * a_direction.y.abs() > b_stub.y * b_direction.y.abs()
                    {
                        builder.line_to(Point::new(
                            a_point.x + halfway_vector.x * a_direction.x,
                            a_point.y - halfway_vector.y * a_direction.y,
                        ));

                        builder.line_to(Point::new(
                            b_point.x + halfway_vector.x * b_direction.x,
                            b_point.y - halfway_vector.y * b_direction.y,
                        ));
                    } else if a_direction.x == 0.0 {
                        builder.line_to(Point::new(a_stub.x + halfway_vector.x, a_stub.y));
                        builder.line_to(Point::new(b_stub.x - halfway_vector.x, b_stub.y));
                    } else {
                        builder.line_to(Point::new(a_stub.x, a_stub.y + halfway_vector.y));
                        builder.line_to(Point::new(b_stub.x, b_stub.y - halfway_vector.y));
                    }

                    builder.line_to(b_stub);

                    builder.line_to(b_point);
                }
                // side with top/bottom
                (Self::SiblingLeft, Self::Parent)
                | (Self::SiblingRight, Self::Parent)
                | (Self::SiblingLeft, Self::Child)
                | (Self::SiblingRight, Self::Child) => {
                    builder.line_to(a_stub);

                    builder.line_to(Point::new(a_stub.x, b_stub.y));

                    builder.line_to(b_stub);
                    builder.line_to(b_point);
                }
                _ => unreachable!(),
            }
        })
    }
}

#[derive(Clone, Debug, Default)]
struct Image {
    path: PathBuf,
}

struct State {
    folder: Option<PathBuf>,
    images: GraphData<Image, Family>,
    panes: pane_grid::State<Pane>,
    assets: assets::AssetsPane,
    focus: Option<pane_grid::Pane>,
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
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    MenuItemPressed(MenuItem),
    AssetsMessage(AssetsMessage),
    FolderOpenFailed(IOError),
    AddImageFailed(IOError),
    NodeDragged(NodeDraggedEvent),
    NodeConnect(OnConnectEvent<Family>),
    NodeDisconnect(usize),
}

fn view(state: &State) -> Element<'_, Message> {
    let file_menu_items = [MenuItem::OpenFolder, MenuItem::AddImage];

    // what a hack lmao
    let a: Option<MenuItem> = None;

    let file_menu = pick_list(file_menu_items, a, Message::MenuItemPressed)
        .placeholder("File")
        .style(|theme, status| {
            let mut style = pick_list::default(theme, status);
            style.border = Border::default();
            let palette = theme.extended_palette();
            style.placeholder_color = palette.background.weak.text;
            style
        });

    let menu_bar = container(row![file_menu].spacing(5.0))
        .width(Fill)
        .style(style::menu_bar);

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
                    .node_attachments(&[
                        Family::Parent,
                        Family::Child,
                        Family::SiblingLeft,
                        Family::SiblingRight,
                    ])
                    .on_connect(Message::NodeConnect)
                    .on_disconnect(Message::NodeDisconnect);

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
    .on_click(Message::Clicked)
    .on_drag(Message::Dragged)
    .on_resize(10, Message::Resized);

    column![menu_bar, container(grid)].into()
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
        Message::AddImageFailed(_) => Task::none(),
        Message::Clicked(pane) => {
            state.focus = Some(pane);
            Task::none()
        }
        Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
            state.panes.resize(split, ratio);
            Task::none()
        }
        Message::Dragged(pane_grid::DragEvent::Dropped { pane, target }) => {
            state.panes.drop(pane, target);
            Task::none()
        }
        Message::Dragged(_) => Task::none(),
        Message::MenuItemPressed(MenuItem::AddImage) => {
            Task::perform(pick_file(), |path_maybe| match path_maybe {
                Ok(path) => Message::AddImage(path),
                Err(err) => Message::AddImageFailed(err),
            })
        }
        Message::MenuItemPressed(MenuItem::OpenFolder) => {
            Task::perform(pick_folder(), |path_maybe| match path_maybe {
                Ok(path) => Message::AssetsMessage(AssetsMessage::LoadAssets(path)),
                Err(err) => Message::FolderOpenFailed(err),
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
        Message::FolderOpenFailed(_err) => Task::none(),
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
    }
}

fn view_image(img: &Image) -> Element<'_, Message> {
    container(
        column![
            image(img.path.clone())
                .width(Fill)
                .height(Fill)
                .filter_method(image::FilterMethod::Nearest),
            text(img.path.file_name().unwrap().to_str().unwrap().to_owned())
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
                    .rounded(5.0)
                    .width(2.0)
                    .color(palette.background.weak.color),
            )
    })
    .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    OpenFolder,
    AddImage,
}

impl std::fmt::Display for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MenuItem::AddImage => "Add Image",
            MenuItem::OpenFolder => "Open Folder",
        })
    }
}
