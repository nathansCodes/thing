mod indicator;
mod provider;
mod receiver;

use iced::{Element, Point};

use crate::widgets::dnd::{
    indicator::DragAndDropIndicator, provider::DragAndDropProvider, receiver::DragAndDropReceiver,
};

pub fn dnd_indicator<'a, Message, Theme, Renderer>(
    payload_element: Option<Element<'a, Message, Theme, Renderer>>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer + 'a,
    Theme: 'a,
{
    Element::from(DragAndDropIndicator {
        payload_element,
        content: content.into(),
    })
}

pub fn dnd_provider<'a, Message, Theme, Renderer>(
    start_dragging: Message,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer + 'a,
    Theme: 'a,
{
    DragAndDropProvider {
        start_dragging,
        content: content.into(),
    }
    .into()
}

pub fn dnd_receiver<'a, Payload, Message, Theme, Renderer, F>(
    receive: F,
    payload: Option<Payload>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    F: Fn(Payload, Point) -> Option<Message> + 'a,
    Payload: Clone + 'a,
    Message: Clone + std::fmt::Debug + 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer + 'a,
    Theme: 'a,
{
    DragAndDropReceiver {
        receive: Box::new(receive),
        payload,
        content: content.into(),
    }
    .into()
}
