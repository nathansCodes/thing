pub mod dnd;
pub mod graph;

use iced::{
    Alignment, Element, Font,
    Length::Fill,
    Padding,
    font::Weight,
    widget::{Container, button, column, container, horizontal_rule, horizontal_space, row, text},
};

use crate::{
    Message,
    notification::Notification,
    style,
    widgets::graph::{Graph, GraphData},
};

pub fn base_button<'a>(content: impl Into<Element<'a, Message>>) -> button::Button<'a, Message> {
    button(content).padding([4, 8]).style(style::base_button)
}

pub fn menu_button(label: &str) -> button::Button<'_, Message> {
    let mut font = Font::DEFAULT;
    font.weight = Weight::Medium;

    base_button(text(label).align_y(Alignment::Center).size(15.0).font(font))
        .on_press(Message::MenuButtonPressed)
        .style(style::menu_item)
}

pub fn menu_item_button<'a>(
    label: &'a str,
    flavor_text: Option<&'a str>,
) -> button::Button<'a, Message> {
    let mut font = Font::DEFAULT;
    font.weight = Weight::Medium;

    base_button(
        row![
            text(label).align_y(Alignment::Center).size(15.0).font(font),
            horizontal_space(),
        ]
        .push_maybe(flavor_text.map(|flavor_text| {
            container(
                text(flavor_text)
                    .align_y(Alignment::Center)
                    .size(13.0)
                    .font(font),
            )
            .align_y(Alignment::Center)
        }))
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .style(style::menu_item)
}

pub fn notification(i: usize, notification: &Notification) -> Container<Message> {
    let mut title_font = Font::DEFAULT;

    title_font.weight = Weight::Bold;

    let mut x_font = Font::DEFAULT;

    x_font.weight = Weight::Black;

    let header = container(
        row![
            text(&notification.title)
                .font(title_font)
                .align_y(Alignment::Center),
            horizontal_space(),
            // TODO: find icons to use
            base_button(
                text("X")
                    .align_y(Alignment::Center)
                    .align_x(Alignment::Center)
                    .font(x_font)
                    .size(14.0)
            )
            .style(style::notification_close_button(&notification.severity))
            .width(25.0)
            .height(25.0)
            .on_press(Message::DismissNotification(i))
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::default().left(8.0).right(2.0).top(2.0).bottom(2.0))
    .style(style::notification_title(&notification.severity));

    container(column![
        header,
        container(
            horizontal_rule(2.0).style(style::notification_timeout_indicator(
                &notification.severity,
                notification.timeout
            ))
        )
        .padding([0.0, 2.0]),
        container(text(&notification.content).size(14.0)).padding(4.0)
    ])
    .width(250.0)
    .style(style::notification(&notification.severity))
}

pub fn graph<'a, Message, Renderer, F, Data, Attachment>(
    data: &'a GraphData<Data, Attachment>,
    view_node: F,
) -> Graph<'a, Message, Renderer, Data, Attachment>
where
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer,
    Data: std::fmt::Debug,
    Attachment: graph::Attachment + PartialEq + 'static,
    F: Fn(&Data) -> Element<Message, iced::Theme, Renderer>,
{
    Graph::new(data, view_node)
}
