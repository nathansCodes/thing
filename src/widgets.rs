use iced::{
    Alignment, Element, Font,
    Length::Fill,
    font::Weight,
    widget::{button, text},
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

pub fn menu_item_button(label: &str) -> button::Button<'_, Message> {
    let mut font = Font::DEFAULT;
    font.weight = Weight::Medium;

    base_button(text(label).align_y(Alignment::Center).size(15.0).font(font))
        .width(Fill)
        .style(style::menu_item)
}
