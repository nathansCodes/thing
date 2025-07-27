pub mod connections;
mod data;

pub use crate::graph::connections::{Attachment, RelativeAttachment};
pub use data::GraphData;

use iced::{
    Border, Color, Element, Event, Length, Padding, Point, Rectangle, Size, Transformation, Vector,
    advanced::{
        Layout, Widget,
        graphics::{core::event::Status, geometry::Frame},
        layout::{Limits, Node},
        renderer::{self, Quad},
        widget::{Tree, tree},
    },
    keyboard::{self, Key, Modifiers},
    mouse::{self, Button, Cursor, ScrollDelta},
    widget::{
        canvas::{LineCap, LineJoin, Path, Stroke},
        container, text,
    },
};

#[allow(clippy::type_complexity)]
pub struct Graph<'a, Message, Theme, Renderer, Data, Attachment = RelativeAttachment>
where
    Theme: container::Catalog,
    Renderer: iced::advanced::image::Renderer,
    Data: std::fmt::Debug,
    Attachment: connections::Attachment,
    Message: 'a,
{
    data: &'a GraphData<Data, Attachment>,
    content: Vec<Element<'a, Message, Theme, Renderer>>,
    on_node_dragged: Option<Box<dyn Fn(NodeDraggedEvent) -> Message + 'a>>,
    get_attachment: Box<dyn Fn(Vector) -> Option<Attachment> + 'a>,
    on_connect: Option<Box<dyn Fn(OnConnectEvent<Attachment>) -> Message + 'a>>,
}

impl<'a, Message, Theme, Renderer, Data, Attachment>
    Graph<'a, Message, Theme, Renderer, Data, Attachment>
where
    Theme: container::Catalog + text::Catalog + 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::text::Renderer + 'a,
    Data: std::fmt::Debug,
    Attachment: connections::Attachment,
    Message: 'a,
{
    pub fn new<F>(data: &'a GraphData<Data, Attachment>, view_node: F) -> Self
    where
        F: Fn(&Data) -> Element<Message, Theme, Renderer>,
    {
        let content = data
            .nodes
            .iter()
            .map(|node| view_node(&node.data))
            .collect();

        Self {
            data,
            content,
            on_node_dragged: None,
            get_attachment: Box::new(|_| None),
            on_connect: None,
        }
    }

    pub fn on_node_dragged<F>(mut self, callback: F) -> Self
    where
        F: 'a + Fn(NodeDraggedEvent) -> Message,
    {
        self.on_node_dragged = Some(Box::new(callback));
        self
    }

    pub fn attachment_under_cursor<F>(mut self, cursor_over_attachment: F) -> Self
    where
        F: Fn(Vector) -> Option<Attachment> + 'a,
    {
        self.get_attachment = Box::new(cursor_over_attachment);
        self
    }

    pub fn node_attachments(mut self, attachments: &'a [Attachment]) -> Self {
        self.get_attachment = Box::new(|relative_cursor_pos| {
            attachments
                .iter()
                .find(|att| {
                    let mut diff = relative_cursor_pos - att.connection_point();

                    diff.x = diff.x.abs();
                    diff.y = diff.y.abs();

                    diff.x < 0.15 && diff.y < 0.15
                })
                .cloned()
        });
        self
    }

    pub fn on_connect<F>(mut self, callback: F) -> Self
    where
        F: 'a + Fn(OnConnectEvent<Attachment>) -> Message,
    {
        self.on_connect = Some(Box::new(callback));
        self
    }
}

pub struct GraphState<Attachment = RelativeAttachment>
where
    Attachment: connections::Attachment,
{
    position: Vector,
    drag_state: DragState<Attachment>,
    drag_origin: Point,
    drag_start_point: Point,
    zoom: f32,
    shift_pressed: bool,
    cursor_pos: Point,
    debug: bool,
}

impl<Attachment: connections::Attachment> Default for GraphState<Attachment> {
    fn default() -> Self {
        Self {
            position: Vector::ZERO,
            drag_state: DragState::None(None),
            drag_origin: Point::ORIGIN,
            drag_start_point: Point::ORIGIN,
            zoom: 1.0,
            shift_pressed: false,
            cursor_pos: Point::ORIGIN,
            debug: false,
        }
    }
}

#[derive(Debug, Clone)]
enum DragState<Attachment = RelativeAttachment>
where
    Attachment: connections::Attachment + std::fmt::Debug,
{
    Node(usize),
    Background,
    None(Option<(usize, Option<Attachment>)>),
    Attachment(usize, Attachment),
}

impl<'a, Message, Theme, Renderer, Data, Attachment> Widget<Message, Theme, Renderer>
    for Graph<'a, Message, Theme, Renderer, Data, Attachment>
where
    Theme: container::Catalog + 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer,
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + 'static,
{
    fn size(&self) -> Size<iced::Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let children = self
            .content
            .iter()
            .zip(tree.children.iter_mut())
            .zip(self.data.nodes.iter().map(|data| data.position))
            .map(|((node, tree), position)| {
                node.as_widget()
                    .layout(tree, renderer, &Limits::NONE)
                    .move_to(position)
            })
            .collect();

        Node::with_children(limits.max(), children)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<GraphState<Attachment>>();

        let bounds_position = layout.bounds().position();

        renderer.with_layer(layout.bounds(), |renderer| {
            renderer.with_translation(
                Vector::new(bounds_position.x, bounds_position.y),
                |renderer| {
                    renderer.with_transformation(Transformation::scale(state.zoom), |renderer| {
                        let mut frame = Frame::new(renderer, layout.bounds().size());

                        // draw connections
                        for connection in &self.data.connections {
                            let a = self.data.get(connection.a.0).unwrap();
                            let b = self.data.get(connection.b.0).unwrap();

                            let a_size = layout
                                .children()
                                .nth(connection.a.0)
                                .expect("Invalid connection")
                                .bounds()
                                .size();

                            let b_size = layout
                                .children()
                                .nth(connection.b.0)
                                .expect("Invalid connection")
                                .bounds()
                                .size();

                            let a_attachment = connection.a.1.clone();
                            let b_attachment = connection.b.1.clone();

                            let from = a.position
                                + state.position
                                + (a_size * a_attachment.connection_point()).into();
                            let to = b.position
                                + state.position
                                + (b_size * b_attachment.connection_point()).into();

                            frame.stroke(
                                &Attachment::path(a_attachment, from, b_attachment, to),
                                Stroke::default()
                                    .with_color(style.text_color)
                                    .with_width(5.0)
                                    .with_line_join(LineJoin::Bevel)
                                    .with_line_cap(LineCap::Round),
                            );
                        }

                        // draw currently dragging attachment line
                        if let DragState::Attachment(i, attachment) = &state.drag_state {
                            let a = self.data.get(*i).unwrap();

                            let a_size = layout
                                .children()
                                .nth(*i)
                                .expect("Invalid connection")
                                .bounds()
                                .size();

                            let from = a.position
                                + state.position
                                + (a_size * attachment.connection_point()).into();

                            let to = (state.cursor_pos
                                - Vector::new(bounds_position.x, bounds_position.y))
                                * Transformation::scale(1.0 / state.zoom);

                            frame.stroke(
                                &Path::line(from, to),
                                Stroke::default()
                                    .with_color(style.text_color)
                                    .with_width(5.0)
                                    .with_line_cap(LineCap::Round),
                            );
                        }

                        renderer.draw_geometry(frame.into_geometry());

                        let bounds = layout.bounds() * Transformation::scale(1.0 / state.zoom);

                        let width = bounds.width;
                        let height = bounds.height;

                        let step_size = match () {
                            () if state.zoom < 0.55 => 60.,
                            () if state.zoom < 0.85 => 30.,
                            () => 15.,
                        };

                        let num_cols = (width / step_size).ceil() as i32;
                        let num_rows = (height / step_size).ceil() as i32;

                        for i in 0..num_cols {
                            for j in 0..num_rows {
                                let x = i as f32 * step_size;
                                let y = j as f32 * step_size;
                                let pos = Point::new(
                                    x + state.position.x % step_size,
                                    y + state.position.y % step_size,
                                );
                                let rect = Rectangle::new(
                                    pos,
                                    Size::new(2.0 / state.zoom, 2.0 / state.zoom),
                                );

                                renderer.fill_quad(
                                    Quad {
                                        bounds: rect,
                                        ..Default::default()
                                    },
                                    style.text_color.scale_alpha(0.3),
                                );
                            }
                        }
                    });
                },
            );
        });

        for (node, tree, node_layout, bounds) in self
            .content
            .iter()
            .zip(tree.children.iter())
            .zip(layout.children())
            .zip(self.data.nodes.iter())
            .filter_map(|(((node, tree), node_layout), data)| {
                transform_node_bounds(
                    node_layout.bounds(),
                    state.zoom,
                    state.position,
                    data.position,
                )
                .intersection(&layout.bounds())
                .map(|bounds| (node, tree, node_layout, bounds))
            })
        {
            renderer.with_layer(bounds, |renderer| {
                renderer.with_transformation(Transformation::scale(state.zoom), |renderer| {
                    let node_pos = Vector::new(layout.position().x, layout.position().y);
                    renderer.with_translation(
                        state.position - node_pos
                            + node_pos * Transformation::scale(1.0 / state.zoom),
                        |renderer| {
                            node.as_widget().draw(
                                tree,
                                renderer,
                                theme,
                                style,
                                node_layout,
                                cursor,
                                &Rectangle::with_size(Size::INFINITY),
                            );
                        },
                    );
                });
            });
        }

        if !state.debug {
            return;
        }

        renderer.fill_quad(
            Quad {
                bounds: Rectangle::new(state.cursor_pos, Size::new(5.0, 5.0)),
                ..Default::default()
            },
            Color::from_rgb(1.0, 0.0, 1.0),
        );

        renderer.with_translation(Vector::new(viewport.x, viewport.y), |renderer| {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(state.drag_origin, Size::new(6.0, 6.0)),
                    ..Default::default()
                },
                Color::from_rgb(1.0, 0.0, 0.0),
            );

            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(state.drag_start_point, Size::new(5.0, 5.0)),
                    ..Default::default()
                },
                Color::from_rgb(0.0, 1.0, 0.0),
            );

            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(
                        state.drag_origin + (state.cursor_pos - state.drag_start_point),
                        Size::new(5.0, 5.0),
                    ),
                    ..Default::default()
                },
                Color::from_rgb(0.0, 1.0, 1.0),
            );
        });

        renderer.fill_quad(
            Quad {
                bounds: Rectangle::new(
                    layout.bounds().center() + state.position,
                    Size::new(5.0, 5.0),
                ),
                ..Default::default()
            },
            Color::from_rgb(0.0, 0.0, 1.0),
        );
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<GraphState<Attachment>>()
    }

    fn state(&self) -> tree::State {
        tree::State::Some(Box::new(GraphState::<Attachment>::default()))
    }

    fn children(&self) -> Vec<Tree> {
        self.content.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.content);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
        let state = tree.state.downcast_mut::<GraphState<Attachment>>();

        let get_hovered_node = || {
            layout
                .children()
                .zip(self.data.nodes.iter())
                .enumerate()
                .find_map(|(i, (child, data))| {
                    match cursor.position().map(|cursor_pos| {
                        let child_bounds = transform_node_bounds(
                            child.bounds(),
                            state.zoom,
                            state.position,
                            data.position,
                        );

                        let hovered = child_bounds.contains(cursor_pos);

                        let mut relative_cursor_pos = cursor_pos - child_bounds.position();

                        relative_cursor_pos.x /= child_bounds.size().width;
                        relative_cursor_pos.y /= child_bounds.size().height;

                        let hovered_attachment = (self.get_attachment)(relative_cursor_pos);

                        if let Some(hovered_attachment) = hovered_attachment {
                            return Some((i, Some(hovered_attachment)));
                        }

                        hovered.then_some((i, None))
                    }) {
                        Some(Some(node)) => Some((node.0, node.1)),
                        _ => None,
                    }
                })
        };

        match event {
            Event::Touch(ev) => {
                dbg!(ev);
                Status::Ignored
            }
            Event::Mouse(mouse::Event::CursorMoved {
                position: cursor_pos,
            }) => {
                state.cursor_pos = cursor_pos;

                match &state.drag_state {
                    DragState::Node(id) => {
                        if let Some(on_node_dragged) = &self.on_node_dragged {
                            let mut new_position = state.drag_origin
                                + (cursor_pos - state.drag_start_point)
                                    * Transformation::scale(1.0 / state.zoom);

                            // limiting the position to positive values cause the
                            // renderer doesn't render elements with negative positions properly
                            new_position.x = new_position.x.max(0.0);
                            new_position.y = new_position.y.max(0.0);

                            shell.publish(on_node_dragged(NodeDraggedEvent {
                                id: *id,
                                new_position,
                            }));
                        }
                    }
                    DragState::Background => {
                        let mut new_position = state.drag_origin
                            + (cursor_pos - state.drag_start_point)
                                * Transformation::scale(1.0 / state.zoom);

                        // same reason as before except that i limit it to negative values for some
                        // reason
                        new_position.x = new_position.x.min(0.0);
                        new_position.y = new_position.y.min(0.0);

                        state.position.x = new_position.x;
                        state.position.y = new_position.y;
                        shell.invalidate_layout();
                    }
                    DragState::None(hovered_node) => match hovered_node {
                        Some((id, _)) => {
                            state.drag_origin = self.data.nodes[*id].position;
                            state.drag_start_point = cursor_pos;
                            state.drag_state = match get_hovered_node() {
                                Some(node) => DragState::None(Some(node)),
                                None => DragState::None(None),
                            };
                        }
                        None => {
                            state.drag_origin.x = state.position.x;
                            state.drag_origin.y = state.position.y;
                            state.drag_start_point = cursor_pos;
                            state.drag_state = match get_hovered_node() {
                                Some(node) => DragState::None(Some(node)),
                                None => DragState::None(None),
                            };
                        }
                    },
                    DragState::Attachment(..) => (),
                }

                Status::Captured
            }
            Event::Mouse(mouse::Event::ButtonPressed(Button::Middle)) => {
                let Some(cursor_pos) = cursor.position() else {
                    return Status::Ignored;
                };

                if !layout.bounds().contains(cursor_pos) {
                    return Status::Ignored;
                }

                state.drag_state = match get_hovered_node() {
                    Some((id, None)) => DragState::Node(id),
                    Some((id, Some(attachment))) => DragState::Attachment(id, attachment),
                    None => DragState::Background,
                };

                Status::Captured
            }
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => {
                let Some(cursor_pos) = cursor.position() else {
                    return Status::Ignored;
                };

                if !layout.bounds().contains(cursor_pos) {
                    return Status::Ignored;
                }

                state.drag_state = match get_hovered_node() {
                    Some((id, None)) if state.shift_pressed => DragState::Node(id),
                    Some((id, Some(attachment))) if state.shift_pressed => {
                        DragState::Attachment(id, attachment)
                    }
                    None if state.shift_pressed => DragState::Background,
                    _ => state.drag_state.clone(),
                };

                Status::Captured
            }
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left | Button::Middle)) => {
                let hovered = get_hovered_node();

                if let Some((b, Some(b_attachment))) = hovered.clone()
                    && let DragState::Attachment(a, a_attachment) = &state.drag_state
                    && let Some(on_connect) = &self.on_connect
                {
                    shell.publish(on_connect(OnConnectEvent::<Attachment> {
                        a: *a,
                        a_attachment: a_attachment.clone(),
                        b,
                        b_attachment,
                    }));
                }

                state.drag_state = DragState::None(hovered);

                Status::Captured
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let Some(cursor_pos) = cursor.position() else {
                    return Status::Ignored;
                };

                if !layout.bounds().contains(cursor_pos) {
                    return Status::Ignored;
                }

                let (delta_x, delta_y) = match delta {
                    ScrollDelta::Lines { x, y } => (x, y * 0.05),
                    ScrollDelta::Pixels { x, y } => (x, y * 0.05),
                };

                state.position.x += delta_x * 5.;

                if state.zoom >= 0.3 && state.zoom <= 2.0 {
                    state.zoom += delta_y;
                    state.zoom = state.zoom.clamp(0.3, 2.0);
                }

                Status::Captured
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(mods)) => {
                state.shift_pressed = mods.contains(Modifiers::SHIFT);

                Status::Captured
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: Key::Character(char),
                location: _,
                modifiers: _,
            }) => {
                if char.eq("d") {
                    state.debug = !state.debug;
                    Status::Captured
                } else {
                    Status::Ignored
                }
            }
            _ => Status::Ignored,
        }
    }
}

impl<'a, Message, Theme, Renderer, Data, Attachment>
    From<Graph<'a, Message, Theme, Renderer, Data, Attachment>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: container::Catalog + 'a,
    Renderer: renderer::Renderer
        + iced::advanced::image::Renderer
        + iced::advanced::graphics::geometry::Renderer
        + 'a,
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + 'static,
{
    fn from(value: Graph<'a, Message, Theme, Renderer, Data, Attachment>) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug)]
pub struct NodeDraggedEvent {
    pub id: usize,
    pub new_position: Point,
}

#[derive(Clone, Debug)]
pub struct OnConnectEvent<Attachment = RelativeAttachment>
where
    Attachment: connections::Attachment,
{
    pub a: usize,
    pub a_attachment: Attachment,
    pub b: usize,
    pub b_attachment: Attachment,
}

fn transform_node_bounds(
    bounds: Rectangle,
    zoom: f32,
    position: Vector,
    node_position: Point,
) -> Rectangle {
    let horizontal_padding = bounds.width * zoom - bounds.width;
    let vertical_padding = bounds.height * zoom - bounds.height;

    (bounds
        * Transformation::translate(position.x * zoom, position.y * zoom)
        * Transformation::translate(-node_position.x, -node_position.y)
        * Transformation::translate(node_position.x * zoom, node_position.y * zoom))
    .expand(Padding {
        top: 0.0,
        right: horizontal_padding,
        bottom: vertical_padding,
        left: 0.0,
    })
}
