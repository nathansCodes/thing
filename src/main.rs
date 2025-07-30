mod assets;
mod file_picker;
mod graph;
mod style;

use crate::graph::{OnConnectEvent, RelativeAttachment, line_styles};

use iced::{
    Alignment, Border, Element, Font,
    Length::Fill,
    Padding, Point, Task, Theme,
    font::Weight,
    widget::{
        button, column, container, image, opaque, pane_grid, pane_grid::Configuration, pick_list,
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
                },
                Task::none(),
            )
        })
}

#[derive(Clone, Debug, Default)]
struct Image {
    path: PathBuf,
}

struct State {
    folder: Option<PathBuf>,
    images: GraphData<Image, RelativeAttachment<line_styles::Bezier>>,
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
    NodeConnect(OnConnectEvent<RelativeAttachment<line_styles::Bezier>>),
    NodeDisconnect(usize),
    ImageButtonPressed,
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
                    .node_attachments(RelativeAttachment::<line_styles::Bezier>::all_edges())
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
        Message::ImageButtonPressed => {
            println!("pressd");
            Task::none()
        }
    }
}

fn view_image(img: &Image) -> Element<'_, Message> {
    container(
        column![
            opaque(
                image(img.path.clone())
                    .width(Fill)
                    .height(Fill)
                    .filter_method(image::FilterMethod::Nearest)
            ),
            text(img.path.file_name().unwrap().to_str().unwrap().to_owned()),
            button("ahkdlfjs").on_press(Message::ImageButtonPressed)
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
