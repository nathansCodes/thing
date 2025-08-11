use iced::{Point, Vector, event::Status, mouse::Button};

use crate::graph::{RelativeAttachment, connections};

pub struct GraphState<Attachment = RelativeAttachment>
where
    Attachment: connections::Attachment,
{
    pub(super) position: Vector,
    pub(super) cursor_state: CursorState<Attachment>,
    pub(super) pressed_mb: Option<Button>,
    pub(super) drag_origin: Point,
    pub(super) drag_start_point: Point,
    pub(super) zoom: f32,
    pub(super) shift_pressed: bool,
    pub(super) cursor_pos: Point,
    pub(super) debug: bool,
    pub(super) selection: Vec<usize>,
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
            selection: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum Payload<Attachment = RelativeAttachment>
where
    Attachment: connections::Attachment + std::fmt::Debug,
{
    Background,
    Node(usize, Status),
    Attachment(usize, Attachment),
    Connection(usize),
    SelectionRect,
}

#[derive(Debug, Clone)]
pub(super) enum CursorState<Attachment = RelativeAttachment>
where
    Attachment: connections::Attachment + std::fmt::Debug,
{
    Hovering(Payload<Attachment>),
    Dragging(Payload<Attachment>),
}
