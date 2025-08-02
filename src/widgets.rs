use iced::{
    Alignment, Element, Font,
    Length::Fill,
    font::Weight,
    widget::{button, container, horizontal_space, row, text},
};

use crate::{Message, style};

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
