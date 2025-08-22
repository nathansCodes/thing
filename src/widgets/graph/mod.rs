pub mod connections;
mod data;
mod iter;
pub mod line_styles;
mod state;

use std::collections::VecDeque;

pub use crate::graph::connections::{Attachment, RelativeAttachment};
use crate::graph::state::{CursorState, GraphState, Payload};
pub use data::{GraphData, GraphNode};

use iced::{
    Border, Color, Element, Event, Gradient, Length, Padding, Point, Rectangle, Size, Theme,
    Transformation, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget,
        graphics::{core::event::Status, geometry::Frame},
        layout::{Limits, Node},
        renderer::{self, Quad},
        widget::{Tree, tree},
    },
    gradient::{ColorStop, Linear},
    keyboard::{self, Key, Modifiers, key::Named},
    mouse::{self, Button, Cursor, ScrollDelta},
    widget::canvas::{LineCap, LineJoin, Path, Stroke},
};
use lyon_algorithms::{
    geom::{
        Angle,
        euclid::{Transform2D, Vector2D},
    },
    raycast::{Ray, raycast_path},
};

#[allow(clippy::type_complexity)]
pub struct Graph<'a, Message, Renderer, Data, Attachment = RelativeAttachment>
where
    Renderer: iced::advanced::image::Renderer,
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + PartialEq,
    Message: 'a,
{
    position: Vector,
    zoom: f32,
    data: &'a GraphData<Data, Attachment>,
    content: Vec<Element<'a, Message, Theme, Renderer>>,
    get_attachment: Box<dyn Fn(&'a GraphNode<Data>, Vector) -> Option<Attachment> + 'a>,
    on_event: Option<Box<dyn Fn(GraphEvent<Attachment>) -> Message + 'a>>,
    node_positioning: Option<
        Box<
            dyn Fn(
                Option<(usize, &GraphNode<Data>, &Attachment, &Attachment, Size<f32>)>,
                usize,
                &'a GraphNode<Data>,
                Size,
                &'a GraphData<Data, Attachment>,
                &Layout,
                Vec<usize>,
            ) -> Vector,
        >,
    >,
    allow_self_connections: bool,
    allow_similar_connections: bool,
}

impl<'a, Message, Renderer, Data, Attachment> Graph<'a, Message, Renderer, Data, Attachment>
where
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer,
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + PartialEq + 'static,
{
    pub(super) fn new<F>(data: &'a GraphData<Data, Attachment>, view_node: F) -> Self
    where
        F: Fn(&'a GraphNode<Data>) -> Element<Message, Theme, Renderer>,
    {
        let content = data.nodes.iter().map(view_node).collect();

        Self {
            position: Vector::ZERO,
            zoom: 1.0,
            data,
            content,
            get_attachment: Box::new(|_, _| None),
            on_event: None,
            allow_self_connections: false,
            allow_similar_connections: false,
            node_positioning: None,
        }
    }

    pub fn position(mut self, position: Vector) -> Self {
        self.position = position;
        self
    }

    pub fn zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    pub fn node_attachments(mut self, attachments: &'a [(Attachment, Vector)]) -> Self {
        self.get_attachment = Box::new(|_data, relative_cursor_pos| {
            attachments
                .iter()
                .find_map(|(att, att_size)| {
                    let mut diff = relative_cursor_pos - att.connection_point();

                    diff.x = diff.x.abs();
                    diff.y = diff.y.abs();

                    (diff.x < att_size.x && diff.y < att_size.y).then_some(att)
                })
                .cloned()
        });
        self
    }

    pub fn per_node_attachments<F, Iter>(mut self, get_attachments: F) -> Self
    where
        F: Fn(&'a Data) -> Iter + 'a,
        Iter: IntoIterator<Item = (Attachment, Vector)> + 'a,
    {
        self.get_attachment = Box::new(move |node, relative_cursor_pos| {
            get_attachments(&node.data)
                .into_iter()
                .find_map(|(att, att_size)| {
                    let mut diff = relative_cursor_pos - att.connection_point();

                    diff.x = diff.x.abs();
                    diff.y = diff.y.abs();

                    (diff.x < att_size.x && diff.y < att_size.y).then_some(att)
                })
        });
        self
    }

    pub fn position_nodes<F>(mut self, f: F) -> Self
    where
        F: Fn(
                Option<(usize, &GraphNode<Data>, &Attachment, &Attachment, Size<f32>)>,
                usize,
                &'a GraphNode<Data>,
                Size,
                &'a GraphData<Data, Attachment>,
                &Layout,
                Vec<usize>,
            ) -> Vector
            + 'static,
    {
        self.node_positioning = Some(Box::new(f));

        self
    }

    pub fn allow_similar_connections(mut self, value: bool) -> Self {
        self.allow_similar_connections = value;
        self
    }

    pub fn allow_self_connections(mut self, value: bool) -> Self {
        self.allow_self_connections = value;
        self
    }

    pub fn on_event<F>(mut self, on_event: F) -> Self
    where
        F: Fn(GraphEvent<Attachment>) -> Message + 'a,
    {
        self.on_event = Some(Box::new(on_event));
        self
    }

    fn find_hovered_connection(&self, cursor_pos: Point, layout: &Layout<'_>) -> Option<usize> {
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

                let from = a_attachment.resolve(a_size, a.position);
                let to = b_attachment.resolve(b_size, b.position);

                let mut path: Vec<_> =
                    Attachment::path(connection.a.1.clone(), from, connection.b.1.clone(), to)
                        .transform(&Transform2D::scale(self.zoom, self.zoom))
                        .raw()
                        .iter()
                        .collect();

                // remove end event so that it doesn't connect the last point with the first one
                path.pop();

                let origin = lyon_algorithms::geom::euclid::Point2D::new(
                    cursor_pos.x - layout.position().x + self.position.x * self.zoom,
                    cursor_pos.y - layout.position().y + self.position.y * self.zoom,
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

    #[allow(clippy::too_many_arguments)]
    fn get_payload(
        &mut self,
        layout: &Layout<'_>,
        tree_children: &mut [Tree],
        cursor: Cursor,
        event: Event,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        status: &mut Status,
        shell: &mut Shell<'_, Message>,
    ) -> Payload<Attachment> {
        self.content
            .iter_mut()
            .zip(layout.children())
            .zip(tree_children.iter_mut())
            .zip(self.data.nodes.iter())
            .enumerate()
            .find_map(
                |(i, (((element, node_layout), tree), node))| match cursor.position() {
                    Some(cursor_pos) => {
                        // make sure the cursor position is transformed properly
                        let cursor = match cursor {
                            Cursor::Unavailable => Cursor::Unavailable,
                            Cursor::Available(point) => Cursor::Available(
                                layout.position()
                                    + (point - layout.position())
                                        * Transformation::scale(1.0 / self.zoom)
                                    + self.position,
                            ),
                        };

                        let node_status = element.as_widget_mut().on_event(
                            tree,
                            event.clone(),
                            node_layout,
                            cursor,
                            renderer,
                            clipboard,
                            shell,
                            &node_layout.bounds(),
                        );

                        if node_status == Status::Captured {
                            *status = Status::Captured;
                        }

                        let child_bounds = transform_node_bounds(
                            node_layout.bounds(),
                            self.zoom,
                            self.position,
                            node.position,
                        );

                        let hovered = child_bounds.contains(cursor_pos);

                        let mut relative_cursor_pos = cursor_pos - child_bounds.position();

                        relative_cursor_pos.x /= child_bounds.size().width;
                        relative_cursor_pos.y /= child_bounds.size().height;

                        let hovered_attachment = (self.get_attachment)(node, relative_cursor_pos);

                        if !hovered && let Some(hovered_attachment) = hovered_attachment {
                            return Some(Payload::Attachment(i, hovered_attachment));
                        }

                        if hovered {
                            Some(Payload::Node(i, node_status))
                        } else {
                            None
                        }
                    }
                    None => None,
                },
            )
            .unwrap_or_else(|| {
                cursor
                    .position()
                    .and_then(|cursor_pos| self.find_hovered_connection(cursor_pos, layout))
                    .map(Payload::Connection)
                    .unwrap_or(Payload::Background)
            })
    }

    fn on_cursor_moved(
        &self,
        cursor_pos: Point,
        state: &mut GraphState<Attachment>,
        mut new_payload: Payload<Attachment>,
        shell: &mut Shell<'_, Message>,
        layout: &Layout<'_>,
    ) -> Status {
        let Some(on_event) = &self.on_event else {
            return Status::Ignored;
        };

        state.cursor_pos = cursor_pos;

        match &state.cursor_state {
            CursorState::Dragging(payload) => match payload {
                Payload::Background => {
                    let mut new_position = state.drag_origin
                        - (cursor_pos - state.drag_start_point)
                            * Transformation::scale(1.0 / self.zoom);

                    // limiting the position to negative values cause the
                    // renderer doesn't render elements with negative positions properly
                    new_position.x = new_position.x.max(0.0);
                    new_position.y = new_position.y.max(0.0);

                    shell.publish(on_event(GraphEvent::Move(new_position)));
                    shell.invalidate_layout();
                }
                Payload::Node(id, status) => {
                    if status == &Status::Ignored
                        && let Some(on_event) = &self.on_event
                        && self.node_positioning.is_none()
                    {
                        let mut new_position = state.drag_origin
                            + (cursor_pos - state.drag_start_point)
                                * Transformation::scale(1.0 / self.zoom);

                        // same reason as before except that i limit it to positive values for some
                        // reason
                        new_position.x = new_position.x.max(0.0);
                        new_position.y = new_position.y.max(0.0);

                        // un-select other nodes if current one isn't part of the selection
                        if !self.data.is_selected(*id).is_ok_and(|selected| selected) {
                            shell.publish(on_event(GraphEvent::ClearSelection));
                        }

                        // make sure no nodes get negative positions
                        let mut correction = Vector::ZERO;

                        let selections_positions: Vec<_> = self
                            .data
                            .selection()
                            .filter_map(|selected: usize| {
                                if selected == *id {
                                    return None;
                                }

                                let node = &self.data.nodes[selected];

                                let new_position =
                                    node.position + (new_position - self.data.nodes[*id].position);

                                if new_position.x < 0.0 {
                                    correction.x = correction.x.max(-new_position.x);
                                }
                                if new_position.y < 0.0 {
                                    correction.y = correction.y.max(-new_position.y);
                                }

                                Some((selected, new_position))
                            })
                            .collect();

                        shell.publish(on_event(GraphEvent::MoveNode {
                            id: *id,
                            new_position: new_position + correction,
                            was_dragged: true,
                        }));

                        selections_positions.iter().for_each(|(id, new_position)| {
                            shell.publish(on_event(GraphEvent::MoveNode {
                                id: *id,
                                new_position: *new_position + correction,
                                was_dragged: true,
                            }));
                        });
                    }
                }
                Payload::Attachment(_, _) => {}
                Payload::Connection(_) => {}
                Payload::SelectionRect => {}
            },
            CursorState::Hovering(payload) => match payload {
                Payload::Background => {
                    state.drag_origin.x = self.position.x;
                    state.drag_origin.y = self.position.y;
                    state.drag_start_point = cursor_pos;
                    state.cursor_state = if state.pressed_mb == Some(Button::Middle) {
                        CursorState::Dragging(new_payload)
                    } else if state.pressed_mb == Some(Button::Left) {
                        CursorState::Dragging(Payload::SelectionRect)
                    } else {
                        CursorState::Hovering(new_payload)
                    };
                }
                Payload::Node(id, status) => 'cursor_state: {
                    let Some(node) = self.data.nodes.get(*id) else {
                        state.cursor_state = CursorState::Hovering(Payload::Background);
                        break 'cursor_state;
                    };
                    state.drag_origin = node.position;
                    state.drag_start_point = cursor_pos;

                    if status == &Status::Captured {
                        new_payload = Payload::Node(*id, Status::Captured);
                    }

                    let mmb_or_shift_lmb_pressed = state.pressed_mb == Some(Button::Middle)
                        || state.pressed_mb == Some(Button::Left) && state.shift_pressed;

                    if status == &Status::Ignored
                        && mmb_or_shift_lmb_pressed
                        && self.node_positioning.is_none()
                    {
                        state.cursor_state = CursorState::Dragging(new_payload);
                    } else {
                        state.cursor_state = CursorState::Hovering(new_payload);
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
                Payload::Connection(connection) if state.pressed_mb == Some(Button::Left) => {
                    if let Some(on_event) = &self.on_event {
                        shell.publish(on_event(GraphEvent::Disconnect {
                            connection_id: *connection,
                        }));
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

                    let a_att_pos = a_attachment.resolve(a_size, a.position - self.position);
                    let b_att_pos = b_attachment.resolve(b_size, b.position - self.position);

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
                Payload::SelectionRect => unreachable!(),
            },
        }

        Status::Captured
    }

    #[allow(clippy::too_many_arguments)]
    fn on_mouse_button_released(
        &self,
        cursor: Cursor,
        btn: mouse::Button,
        state: &mut GraphState<Attachment>,
        new_payload: Payload<Attachment>,
        shell: &mut Shell<'_, Message>,
        layout: Layout<'_>,
        status: Status,
        viewport: &Rectangle,
    ) -> Status {
        // i love you rust
        let (Button::Left | Button::Middle) = btn else {
            return Status::Ignored;
        };

        state.pressed_mb = None;

        match &state.cursor_state {
            CursorState::Dragging(Payload::Attachment(a, a_attachment)) => {
                if let Some(on_event) = &self.on_event {
                    if let Payload::Attachment(b, b_attachment) = new_payload.clone() {
                        let mut allowed = true;

                        if !self.allow_self_connections && *a == b {
                            allowed = false;
                        }

                        if !self.allow_similar_connections
                            && let Some((connection_id, _)) =
                                self.data.connections.iter().enumerate().find(|(_, conn)| {
                                    (conn.a.0 == *a && conn.b.0 == b)
                                        || (conn.a.0 == b && conn.b.0 == *a)
                                })
                        {
                            shell.publish(on_event(GraphEvent::Disconnect { connection_id }));
                        }

                        if allowed {
                            shell.publish(on_event(GraphEvent::Connect {
                                a: *a,
                                a_attachment: a_attachment.clone(),
                                b,
                                b_attachment,
                            }));
                        }
                    } else if matches!(new_payload, Payload::Background) {
                        shell.publish(on_event(GraphEvent::ConnectionDropped {
                            id: *a,
                            attachment: a_attachment.clone(),
                        }))
                    }
                }
            }
            CursorState::Dragging(Payload::SelectionRect) => {
                let mut top_left = state.drag_start_point;
                let mut bottom_right = state.cursor_pos;

                if bottom_right.x < top_left.x {
                    std::mem::swap(&mut top_left.x, &mut bottom_right.x);
                }
                if bottom_right.y < top_left.y {
                    std::mem::swap(&mut top_left.y, &mut bottom_right.y);
                }

                let rect = Rectangle::new(top_left, (bottom_right - top_left).into());

                let selected: Vec<_> = layout
                    .children()
                    .zip(self.data.nodes.iter())
                    .enumerate()
                    .filter_map(|(i, (child, node))| {
                        rect.intersects(&transform_node_bounds(
                            child.bounds(),
                            self.zoom,
                            self.position,
                            node.position,
                        ))
                        .then_some(i)
                    })
                    .collect();

                if let Some(on_event) = &self.on_event {
                    if !state.shift_pressed {
                        shell.publish(on_event(GraphEvent::ClearSelection));
                    }

                    for id in &selected {
                        shell.publish(on_event(GraphEvent::Select(*id)));
                    }
                }
            }
            CursorState::Hovering(Payload::Node(id, old_status))
                if viewport
                    .intersection(&layout.bounds())
                    .is_some_and(|bounds| cursor.is_over(bounds))
                    && old_status == &Status::Ignored
                    && status == Status::Ignored =>
            {
                if let Some(on_event) = &self.on_event {
                    if !state.shift_pressed {
                        shell.publish(on_event(GraphEvent::ClearSelection));
                    }
                    if let Some(index) = self.data.selection().position(|s| s == *id) {
                        shell.publish(on_event(GraphEvent::Deselect(index)));
                    } else {
                        shell.publish(on_event(GraphEvent::Select(*id)));
                    }
                }
            }
            CursorState::Hovering(Payload::Background)
                if viewport
                    .intersection(&layout.bounds())
                    .is_some_and(|bounds| cursor.is_over(bounds)) =>
            {
                if let Some(on_event) = &self.on_event {
                    shell.publish(on_event(GraphEvent::ClearSelection));
                }
            }
            _ => (),
        }

        state.cursor_state = CursorState::Hovering(new_payload);

        Status::Captured
    }
}

impl<'a, Message, Renderer, Data, Attachment> Widget<Message, Theme, Renderer>
    for Graph<'a, Message, Renderer, Data, Attachment>
where
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer,
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + PartialEq + 'static,
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
        let palette = theme.extended_palette();

        let bounds_position = layout.bounds().position();

        renderer.with_layer(layout.bounds(), |renderer| {
            renderer.with_translation(
                Vector::new(bounds_position.x, bounds_position.y),
                |renderer| {
                    renderer.with_transformation(Transformation::scale(self.zoom), |renderer| {
                        let mut frame =
                            Frame::new(renderer, layout.bounds().size() * (1.0 / self.zoom));

                        // draw connections
                        for (i, connection) in self.data.connections.iter().enumerate() {
                            let a = self.data.get(connection.a.0).unwrap();
                            let b = self.data.get(connection.b.0).unwrap();

                            let a_size = layout
                                .children()
                                .nth(connection.a.0)
                                .unwrap_or_else(|| {
                                    dbg!(&self.data);

                                    panic!(
                                        "Invalid connection: {:?}\nnode not found: {}",
                                        connection, connection.a.0
                                    )
                                })
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

                            let from = a_attachment.resolve(a_size, a.position - self.position);
                            let to = b_attachment.resolve(b_size, b.position - self.position);

                            let path = Attachment::path(a_attachment, from, b_attachment, to);

                            frame.stroke(
                                &path,
                                Stroke::default()
                                    .with_color(palette.secondary.strong.color)
                                    .with_width(5.0 / self.zoom)
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
                                        .with_color(palette.secondary.strong.color.scale_alpha(0.3))
                                        .with_width(10.0 / self.zoom)
                                        .with_line_join(LineJoin::Bevel)
                                        .with_line_cap(LineCap::Round),
                                );
                            }
                        }

                        if let CursorState::Hovering(Payload::Attachment(i, attachment)) =
                            &state.cursor_state
                        {
                            let node = self.data.get(*i).unwrap();

                            let size = layout
                                .children()
                                .nth(*i)
                                .expect("Invalid connection")
                                .bounds()
                                .size();

                            let attachment_point =
                                attachment.resolve(size, node.position - self.position);

                            frame.fill(
                                &Path::circle(attachment_point, 10.0 / self.zoom),
                                palette.primary.strong.color,
                            );
                        }

                        // draw currently dragging attachment line
                        if let CursorState::Dragging(Payload::Attachment(i, attachment)) =
                            &state.cursor_state
                        {
                            let node = self.data.get(*i).unwrap();

                            let size = layout
                                .children()
                                .nth(*i)
                                .expect("Invalid connection")
                                .bounds()
                                .size();

                            let from = attachment.resolve(size, node.position - self.position);

                            let to = (state.cursor_pos
                                - Vector::new(bounds_position.x, bounds_position.y))
                                * Transformation::scale(1.0 / self.zoom);

                            frame.stroke(
                                &Path::line(from, to),
                                Stroke::default()
                                    .with_color(palette.secondary.strong.color)
                                    .with_width(5.0 / self.zoom)
                                    .with_line_cap(LineCap::Round),
                            );
                            frame.fill(
                                &Path::circle(from, 7.5 / self.zoom),
                                palette.primary.strong.color,
                            );
                        }

                        renderer.draw_geometry(frame.into_geometry());

                        let bounds = layout.bounds() * Transformation::scale(1.0 / self.zoom);

                        let width = bounds.width;
                        let height = bounds.height;

                        let step_size = match () {
                            () if self.zoom < 0.55 => 60.,
                            () if self.zoom < 0.85 => 30.,
                            () => 15.,
                        };

                        let num_cols = (width / step_size).ceil() as i32;
                        let num_rows = (height / step_size).ceil() as i32;

                        for i in 0..num_cols {
                            for j in 0..num_rows {
                                let x = i as f32 * step_size;
                                let y = j as f32 * step_size;
                                let pos = Point::new(
                                    x - self.position.x % step_size,
                                    y - self.position.y % step_size,
                                );
                                let rect = Rectangle::new(
                                    pos,
                                    Size::new(2.0 / self.zoom, 2.0 / self.zoom),
                                );

                                renderer.fill_quad(
                                    Quad {
                                        bounds: rect,
                                        ..Default::default()
                                    },
                                    palette.background.base.text.scale_alpha(0.3),
                                );
                            }
                        }
                    });
                },
            );
        });

        self.content
            .iter()
            .zip(tree.children.iter())
            .zip(layout.children())
            .zip(self.data.nodes.iter())
            .filter_map(|(((element, tree), node_layout), data)| {
                transform_node_bounds(
                    node_layout.bounds(),
                    self.zoom,
                    self.position,
                    data.position,
                )
                .intersection(&layout.bounds())
                .map(|bounds| (element, tree, node_layout, bounds))
            })
            .for_each(|(node, tree, node_layout, bounds)| {
                renderer.with_layer(bounds, |renderer| {
                    renderer.with_transformation(Transformation::scale(self.zoom), |renderer| {
                        let node_pos = Vector::new(layout.position().x, layout.position().y);

                        renderer.with_translation(
                            Vector::ZERO - node_pos + node_pos * (1.0 / self.zoom) - self.position,
                            |renderer| {
                                // make sure the cursor position is transformed properly
                                let cursor = match cursor {
                                    Cursor::Unavailable => Cursor::Unavailable,
                                    Cursor::Available(point) => Cursor::Available(
                                        layout.position()
                                            + (point - layout.position())
                                                * Transformation::scale(1.0 / self.zoom)
                                            + self.position,
                                    ),
                                };

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
            });

        renderer.with_layer(layout.bounds(), |renderer| {
            renderer.fill_quad(
                Quad {
                    bounds: layout.bounds(),
                    ..Default::default()
                },
                Gradient::Linear(
                    Linear::new(std::f32::consts::PI / 2.0).add_stops([
                        ColorStop {
                            offset: 0.0,
                            color: palette
                                .background
                                .base
                                .color
                                .scale_alpha(self.position.x.min(10.0) / 10.0),
                        },
                        ColorStop {
                            offset: 10.0 / layout.bounds().width,
                            color: palette.background.base.color.scale_alpha(0.0),
                        },
                        ColorStop {
                            offset: 1.0 - 10.0 / layout.bounds().width,
                            color: palette.background.base.color.scale_alpha(0.0),
                        },
                        ColorStop {
                            offset: 1.0,
                            color: palette.background.base.color,
                        },
                    ]),
                ),
            );
            renderer.fill_quad(
                Quad {
                    bounds: layout.bounds(),
                    ..Default::default()
                },
                Gradient::Linear(
                    Linear::new(0.0).add_stops([
                        ColorStop {
                            offset: 0.0,
                            color: palette.background.base.color,
                        },
                        ColorStop {
                            offset: 10.0 / layout.bounds().width,
                            color: palette.background.base.color.scale_alpha(0.0),
                        },
                        ColorStop {
                            offset: 1.0 - 10.0 / layout.bounds().width,
                            color: palette.background.base.color.scale_alpha(0.0),
                        },
                        ColorStop {
                            offset: 1.0,
                            color: palette
                                .background
                                .base
                                .color
                                .scale_alpha(self.position.y.min(10.0) / 10.0),
                        },
                    ]),
                ),
            );
        });

        if let CursorState::Dragging(Payload::SelectionRect) = &state.cursor_state {
            renderer.with_layer(layout.bounds(), |renderer| {
                let mut top_left = state.drag_start_point;
                let mut bottom_right = state.cursor_pos;

                if bottom_right.x < top_left.x {
                    std::mem::swap(&mut top_left.x, &mut bottom_right.x);
                }
                if bottom_right.y < top_left.y {
                    std::mem::swap(&mut top_left.y, &mut bottom_right.y);
                }

                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(top_left, (bottom_right - top_left).into()),
                        border: Border::default()
                            .width(3.0)
                            .rounded(10.0)
                            .color(palette.primary.base.color),
                        ..Default::default()
                    },
                    palette.primary.base.color.scale_alpha(0.1),
                );
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
                    layout.bounds().center() - self.position,
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
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> Status {
        let state = tree.state.downcast_mut::<GraphState<Attachment>>();

        let mut status = Status::Ignored;

        let new_payload = self.get_payload(
            &layout,
            &mut tree.children,
            cursor,
            event.clone(),
            renderer,
            clipboard,
            &mut status,
            shell,
        );

        let Some(on_event) = &self.on_event else {
            return Status::Ignored;
        };

        let new_status: Status = match event {
            Event::Touch(ev) => {
                dbg!(ev);
                Status::Ignored
            }
            Event::Mouse(mouse::Event::CursorMoved {
                position: cursor_pos,
            }) => self.on_cursor_moved(cursor_pos, state, new_payload, shell, &layout),
            Event::Mouse(mouse::Event::ButtonPressed(button)) => 'ev: {
                let Some(cursor_pos) = cursor.position() else {
                    break 'ev Status::Ignored;
                };

                if !layout.bounds().contains(cursor_pos) {
                    break 'ev Status::Ignored;
                }

                state.pressed_mb = Some(button);

                state.cursor_state = CursorState::Hovering(new_payload);

                Status::Captured
            }
            Event::Mouse(mouse::Event::ButtonReleased(btn)) => self.on_mouse_button_released(
                cursor,
                btn,
                state,
                new_payload,
                shell,
                layout,
                status,
                viewport,
            ),
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => 'ev: {
                let Some(cursor_pos) = cursor.position() else {
                    break 'ev Status::Ignored;
                };

                if !layout.bounds().contains(cursor_pos) {
                    break 'ev Status::Ignored;
                }

                let (delta_x, delta_y) = match delta {
                    ScrollDelta::Lines { x, y } => (x, y * 0.05),
                    ScrollDelta::Pixels { x, y } => (x, y * 0.05),
                };

                if delta_x != 0.0 {
                    shell.publish(on_event(GraphEvent::Move(Point::new(
                        (self.position.x + delta_x * 5.0 / self.zoom).min(0.0),
                        self.position.y,
                    ))));
                }

                let new_zoom = self.zoom + delta_y;

                if (0.5..=2.0).contains(&self.zoom) || (0.5..=2.0).contains(&new_zoom) {
                    shell.publish(on_event(GraphEvent::Zoom(new_zoom.clamp(0.5, 2.0))));
                }

                Status::Captured
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::Delete),
                ..
            }) => {
                if let Some(on_event) = &self.on_event {
                    for (selection_index, selected_node_id) in self.data.selection().enumerate() {
                        // find how many of the previous selections have an id less than the
                        // current one and subtract the current one from that. we don't want an
                        // an out of bounds memory access
                        let correction = self
                            .data
                            .selection()
                            .take(selection_index)
                            .filter(|s| *s < selected_node_id)
                            .count();

                        let corrected_node_id = selected_node_id - correction.min(selection_index);

                        // disconnect
                        for (connection_id, _, _, _) in
                            self.data.get_connections_indexed(corrected_node_id)
                        {
                            on_event(GraphEvent::Disconnect { connection_id });
                        }

                        match new_payload {
                            Payload::Node(id, _) | Payload::Attachment(id, _)
                                if id == selected_node_id =>
                            {
                                state.cursor_state = CursorState::Hovering(Payload::Background)
                            }
                            Payload::Connection(id)
                                if self.data.connections[id].a.0 == selected_node_id
                                    || self.data.connections[id].b.0 == selected_node_id =>
                            {
                                state.cursor_state = CursorState::Hovering(Payload::Background);
                            }
                            _ => (),
                        }

                        shell.publish(on_event(GraphEvent::Delete {
                            id: corrected_node_id,
                        }));
                    }

                    shell.publish(on_event(GraphEvent::ClearSelection));
                    Status::Captured
                } else {
                    Status::Ignored
                }
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(mods)) => {
                state.shift_pressed = mods.contains(Modifiers::SHIFT);

                Status::Captured
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Character(char),
                modifiers,
                ..
            }) => match char.chars().next().unwrap() {
                'd' if modifiers.control() => {
                    state.debug = !state.debug;
                    Status::Captured
                }
                _ => Status::Ignored,
            },
            _ => Status::Ignored,
        };

        if new_status == Status::Captured {
            status = Status::Captured;
        }

        if let Some(node_positioning) = &self.node_positioning
            && let Some(on_event) = &self.on_event
            && status == Status::Captured
            && !self.data.nodes.is_empty()
        {
            let num_nodes = self.data.nodes.len();
            let mut queue = VecDeque::with_capacity(num_nodes);
            let mut visited = Vec::with_capacity(num_nodes);

            queue.push_back(0);
            visited.push(0);

            let first_pos = Point::ORIGIN
                + node_positioning(
                    None,
                    0,
                    &self.data.nodes[0],
                    layout.children().next().unwrap().bounds().size(),
                    self.data,
                    &layout,
                    visited.clone(),
                );

            let mut positions = Vec::from([first_pos]);
            positions.reserve_exact(num_nodes - 1);

            let mut correction = Vector::ZERO;
            let mut last_corrected_node = 0;

            while !queue.is_empty() {
                if visited.len() == self.data.nodes.len() {
                    break;
                }

                let current_node = *queue.front().unwrap();
                let current_pos =
                    positions[visited.iter().position(|id| *id == current_node).unwrap()];

                let unvisited_connections: Vec<_> = self
                    .data
                    .connections
                    .iter()
                    .filter_map(|conn| {
                        (conn.a.0 == current_node)
                            .then_some((conn.b.0, &conn.b.1, &conn.a.1))
                            .or_else(|| {
                                (conn.b.0 == current_node)
                                    .then_some((conn.a.0, &conn.a.1, &conn.b.1))
                            })
                            .filter(|conn| !visited.contains(&conn.0))
                    })
                    .collect();

                queue.extend(unvisited_connections.iter().map(|(id, ..)| *id));

                for (node, attachment, conn_attachment) in &unvisited_connections {
                    let prev_position = current_pos;

                    let new_position = prev_position
                        + node_positioning(
                            Some((
                                current_node,
                                &self.data.nodes[current_node],
                                attachment,
                                conn_attachment,
                                layout.children().nth(current_node).unwrap().bounds().size(),
                            )),
                            *node,
                            &self.data.nodes[*node],
                            layout.children().nth(*node).unwrap().bounds().size(),
                            self.data,
                            &layout,
                            visited.clone(),
                        );

                    visited.push(*node);

                    positions.push(new_position);

                    if new_position.x < correction.x {
                        correction.x = new_position.x;
                    }
                    if new_position.y < correction.y {
                        correction.y = new_position.y;
                    }
                }

                queue.pop_front();

                if queue.is_empty() {
                    for position in positions.iter_mut().skip(last_corrected_node) {
                        *position = *position - correction;
                    }
                    last_corrected_node = positions.len();

                    correction = Vector::ZERO;

                    if let Some((next, node)) = self
                        .data
                        .nodes
                        .iter()
                        .enumerate()
                        .find(|(id, _)| !visited.contains(id))
                    {
                        visited.push(next);

                        let new_position = Point::ORIGIN
                            + node_positioning(
                                None,
                                next,
                                node,
                                layout.children().nth(next).unwrap().bounds().size(),
                                self.data,
                                &layout,
                                visited.clone(),
                            );

                        if new_position.x < 0.0 {
                            correction.x = new_position.x;
                        }
                        if new_position.y < 0.0 {
                            correction.y = new_position.y;
                        }

                        positions.push(new_position);
                        queue.push_back(next);
                    }
                }
            }

            for position in positions.iter_mut().skip(last_corrected_node) {
                *position = *position - correction;
            }

            for (id, new_position) in visited.iter().zip(positions) {
                shell.publish(on_event(GraphEvent::MoveNode {
                    id: *id,
                    new_position,
                    was_dragged: false,
                }));
            }
        }

        status
    }
}

impl<'a, Message, Renderer, Data, Attachment> From<Graph<'a, Message, Renderer, Data, Attachment>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: renderer::Renderer
        + iced::advanced::image::Renderer
        + iced::advanced::graphics::geometry::Renderer
        + 'a,
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + PartialEq + 'static,
{
    fn from(value: Graph<'a, Message, Renderer, Data, Attachment>) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug)]
pub enum GraphEvent<Attachment = RelativeAttachment>
where
    Attachment: connections::Attachment,
{
    Move(Point),
    Zoom(f32),
    MoveNode {
        id: usize,
        new_position: Point,
        was_dragged: bool,
    },
    Connect {
        a: usize,
        a_attachment: Attachment,
        b: usize,
        b_attachment: Attachment,
    },
    Disconnect {
        connection_id: usize,
    },
    Delete {
        id: usize,
    },
    ConnectionDropped {
        id: usize,
        attachment: Attachment,
    },
    Select(usize),
    Deselect(usize),
    ClearSelection,
    SelectAll,
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
        * Transformation::translate(-position.x * zoom, -position.y * zoom)
        * Transformation::translate(-node_position.x, -node_position.y)
        * Transformation::translate(node_position.x * zoom, node_position.y * zoom))
    .expand(Padding {
        top: 0.0,
        right: horizontal_padding,
        bottom: vertical_padding,
        left: 0.0,
    })
}
