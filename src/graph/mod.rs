pub mod connections;
mod data;

pub use crate::graph::connections::{Attachment, RelativeAttachment};
pub use data::GraphData;

use iced::{
    Color, Element, Event, Length, Padding, Point, Rectangle, Size, Transformation, Vector,
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
        canvas::{Fill, LineCap, LineJoin, Path, Stroke, Style, fill::Rule},
        container,
    },
};
use lyon_algorithms::{
    geom::{
        Angle,
        euclid::{Transform2D, Vector2D},
    },
    raycast::{Ray, raycast_path},
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
    on_disconnect: Option<Box<dyn Fn(usize) -> Message + 'a>>,
}

impl<'a, Message, Theme, Renderer, Data, Attachment>
    Graph<'a, Message, Theme, Renderer, Data, Attachment>
where
    Theme: container::Catalog + 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer,
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + 'static,
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
            on_disconnect: None,
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

    pub fn on_disconnect<F>(mut self, callback: F) -> Self
    where
        F: 'a + Fn(usize) -> Message,
    {
        self.on_disconnect = Some(Box::new(callback));
        self
    }

    fn find_hovered_connection(
        &self,
        state: &GraphState<Attachment>,
        cursor_pos: Point,
        layout: &Layout<'_>,
    ) -> Option<usize> {
        self.data
            .connections
            .iter()
            .enumerate()
            .find_map(|(i, connection)| {
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

                let from = a_attachment.resolve(a_size, a.position + state.position);
                let to = b_attachment.resolve(b_size, b.position + state.position);

                let mut path: Vec<_> =
                    Attachment::path(connection.a.1.clone(), from, connection.b.1.clone(), to)
                        .transform(&Transform2D::scale(state.zoom, state.zoom))
                        .raw()
                        .iter()
                        .collect();

                // remove end event so that it doesn't connect the last point with the first one
                path.pop();

                let origin = lyon_algorithms::geom::euclid::Point2D::new(
                    cursor_pos.x - layout.position().x - state.position.x,
                    cursor_pos.y - layout.position().y - state.position.y,
                );

                let directions: Vec<_> = [
                    Angle::zero(),
                    Angle::pi() / 2.0,
                    Angle::pi(),
                    (Angle::pi() * 3.0) / 2.0,
                ]
                .iter()
                .map(|angle| Vector2D::from_angle_and_length(*angle, 1.0))
                .collect();

                let hit = directions.iter().any(|direction| {
                    raycast_path(
                        &Ray {
                            origin,
                            direction: *direction,
                        },
                        path.clone(),
                        15.0,
                    )
                    .is_some_and(|hit| hit.position.distance_to(origin) < 10.0)
                });

                if hit {
                    return Some(i);
                }
                None
            })
    }
}

pub struct GraphState<Attachment = RelativeAttachment>
where
    Attachment: connections::Attachment,
{
    position: Vector,
    cursor_state: CursorState<Attachment>,
    pressed_mb: Option<Button>,
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
            cursor_state: CursorState::Hovering(Payload::Background),
            pressed_mb: None,
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
enum Payload<Attachment = RelativeAttachment>
where
    Attachment: connections::Attachment + std::fmt::Debug,
{
    Background,
    Node(usize),
    Attachment(usize, Attachment),
    Connection(usize),
}

#[derive(Debug, Clone)]
enum CursorState<Attachment = RelativeAttachment>
where
    Attachment: connections::Attachment + std::fmt::Debug,
{
    Hovering(Payload<Attachment>),
    Dragging(Payload<Attachment>),
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
                        let mut frame =
                            Frame::new(renderer, layout.bounds().size() * (1.0 / state.zoom));

                        // draw connections
                        for (i, connection) in self.data.connections.iter().enumerate() {
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

                            let path = Attachment::path(a_attachment, from, b_attachment, to);

                            frame.stroke(
                                &path,
                                Stroke::default()
                                    .with_color(style.text_color)
                                    .with_width(5.0 / state.zoom)
                                    .with_line_join(LineJoin::Bevel)
                                    .with_line_cap(LineCap::Round),
                            );

                            if let CursorState::Hovering(Payload::Connection(j)) =
                                &state.cursor_state
                                && *j == i
                            {
                                frame.stroke(
                                    &path,
                                    Stroke::default()
                                        .with_color(style.text_color.scale_alpha(0.3))
                                        .with_width(10.0 / state.zoom)
                                        .with_line_join(LineJoin::Bevel)
                                        .with_line_cap(LineCap::Round),
                                );
                            }
                        }

                        // draw currently dragging attachment line
                        if let CursorState::Dragging(Payload::Attachment(i, attachment)) =
                            &state.cursor_state
                        {
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
                                    .with_width(5.0 / state.zoom)
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

        let new_payload = layout
            .children()
            .zip(self.data.nodes.iter())
            .enumerate()
            .find_map(|(i, (child, data))| match cursor.position() {
                Some(cursor_pos) => {
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
                        return Some(Payload::Attachment(i, hovered_attachment));
                    }

                    if hovered {
                        Some(Payload::Node(i))
                    } else {
                        None
                    }
                }
                None => None,
            })
            .unwrap_or_else(|| {
                cursor
                    .position()
                    .and_then(|cursor_pos| self.find_hovered_connection(state, cursor_pos, &layout))
                    .map(Payload::Connection)
                    .unwrap_or(Payload::Background)
            });

        match event {
            Event::Touch(ev) => {
                dbg!(ev);
                Status::Ignored
            }
            Event::Mouse(mouse::Event::CursorMoved {
                position: cursor_pos,
            }) => {
                state.cursor_pos = cursor_pos;

                match &state.cursor_state {
                    CursorState::Dragging(payload) => match payload {
                        Payload::Background => {
                            let mut new_position = state.drag_origin
                                + (cursor_pos - state.drag_start_point)
                                    * Transformation::scale(1.0 / state.zoom);

                            // limiting the position to negative values cause the
                            // renderer doesn't render elements with negative positions properly
                            new_position.x = new_position.x.min(0.0);
                            new_position.y = new_position.y.min(0.0);

                            state.position.x = new_position.x;
                            state.position.y = new_position.y;
                            shell.invalidate_layout();
                        }
                        Payload::Node(id) => {
                            if let Some(on_node_dragged) = &self.on_node_dragged {
                                let mut new_position = state.drag_origin
                                    + (cursor_pos - state.drag_start_point)
                                        * Transformation::scale(1.0 / state.zoom);

                                // same reason as before except that i limit it to positive values for some
                                // reason
                                new_position.x = new_position.x.max(0.0);
                                new_position.y = new_position.y.max(0.0);

                                shell.publish(on_node_dragged(NodeDraggedEvent {
                                    id: *id,
                                    new_position,
                                }));
                            }
                        }
                        Payload::Attachment(_, _) => {}
                        Payload::Connection(_) => {}
                    },
                    CursorState::Hovering(payload) => match payload {
                        Payload::Background => {
                            state.drag_origin.x = state.position.x;
                            state.drag_origin.y = state.position.y;
                            state.drag_start_point = cursor_pos;
                            state.cursor_state = if state.pressed_mb == Some(Button::Middle) {
                                CursorState::Dragging(new_payload)
                            } else {
                                CursorState::Hovering(new_payload)
                            };
                        }
                        Payload::Node(id) => {
                            state.drag_origin = self.data.nodes[*id].position;
                            state.drag_start_point = cursor_pos;
                            state.cursor_state = if state.pressed_mb == Some(Button::Middle) {
                                CursorState::Dragging(new_payload)
                            } else {
                                CursorState::Hovering(new_payload)
                            };
                        }
                        Payload::Attachment(id, _) => {
                            state.drag_origin = self.data.nodes[*id].position;
                            state.drag_start_point = cursor_pos;
                            state.cursor_state = if state.pressed_mb == Some(Button::Left) {
                                CursorState::Dragging(new_payload)
                            } else {
                                CursorState::Hovering(new_payload)
                            };
                        }
                        Payload::Connection(connection)
                            if state.pressed_mb == Some(Button::Left) =>
                        {
                            if let Some(on_disconnect) = &self.on_disconnect {
                                shell.publish(on_disconnect(*connection));
                            }

                            let connection = self
                                .data
                                .connections
                                .get(*connection)
                                .expect("Invalid connection");

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

                            let a_att_pos =
                                a_attachment.resolve(a_size, a.position + state.position);
                            let b_att_pos =
                                b_attachment.resolve(b_size, b.position + state.position);

                            let cursor_pos =
                                cursor_pos - Vector::new(layout.position().x, layout.position().y);

                            let delta_to_a = a_att_pos.distance(cursor_pos);
                            let delta_to_b = b_att_pos.distance(cursor_pos);

                            let new_payload = if delta_to_a < delta_to_b {
                                Payload::Attachment(connection.b.0, b_attachment)
                            } else {
                                Payload::Attachment(connection.a.0, a_attachment)
                            };

                            state.cursor_state = CursorState::Dragging(new_payload);
                        }
                        Payload::Connection(_) => {
                            state.cursor_state = CursorState::Hovering(new_payload);
                        }
                    },
                }

                Status::Captured
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                let Some(cursor_pos) = cursor.position() else {
                    return Status::Ignored;
                };

                if !layout.bounds().contains(cursor_pos) {
                    return Status::Ignored;
                }

                state.pressed_mb = Some(button);

                state.cursor_state = CursorState::Hovering(new_payload);

                Status::Captured
            }
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left | Button::Middle)) => {
                state.pressed_mb = None;

                if let Payload::Attachment(b, b_attachment) = new_payload.clone()
                    && let CursorState::Dragging(Payload::Attachment(a, a_attachment)) =
                        &state.cursor_state
                    && let Some(on_connect) = &self.on_connect
                {
                    shell.publish(on_connect(OnConnectEvent::<Attachment> {
                        a: *a,
                        a_attachment: a_attachment.clone(),
                        b,
                        b_attachment,
                    }));
                }

                state.cursor_state = CursorState::Hovering(new_payload);

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
