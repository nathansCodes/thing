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

pub fn dnd_provider<'a, Payload, Message, Theme, Renderer>(
    set_payload: impl Fn(Option<Payload>) -> Message + 'a,
    payload: Payload,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Element<'a, Message, Theme, Renderer>
where
    Payload: Clone + 'a,
    Message: Clone + 'a,
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer + 'a,
    Theme: 'a,
{
    DragAndDropProvider {
        set_payload: Box::new(set_payload),
        payload,
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
