pub mod dialog;
pub mod dnd;
pub mod graph;
#[allow(unused)]
pub mod icons;

pub use dialog::dialog;
use iced::{
    Alignment, Border, Element, Font,
    Length::Fill,
    Padding, Theme,
    advanced::widget::Text,
    font::Weight,
    widget::{
        Container, button, column, container, horizontal_rule, horizontal_space, image, mouse_area,
        opaque, row, scrollable, text,
    },
};
use iced_aw::DropDown;

use crate::{
    Message, Node,
    assets::{AssetsData, Character, Image, image::default_image},
    notification::Notification,
    style,
    widgets::graph::{Graph, GraphData, GraphNode},
};

pub fn base_button<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> button::Button<'a, Message> {
    button(content).padding([4, 8]).style(style::primary_button)
}

pub fn menu_button<'a, Message>(label: &'a str, message: Message) -> button::Button<'a, Message> {
    let mut font = Font::DEFAULT;
    font.weight = Weight::Medium;

    base_button(text(label).align_y(Alignment::Center).size(15.0).font(font))
        .on_press(message)
        .style(style::menu_button)
}

pub fn menu_item_button<'a, Message: 'a>(
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
            text(flavor_text)
                .align_y(Alignment::Center)
                .size(13.0)
                .font(font)
        }))
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .style(style::menu_button)
}

pub fn notification(i: usize, notification: &Notification) -> Container<'_, Message> {
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
            base_button(
                icons::close()
                    .align_y(Alignment::Center)
                    .align_x(Alignment::Center)
                    .size(15.0)
            )
            .style(style::notification_close_button(notification.severity))
            .width(25.0)
            .height(25.0)
            .on_press(Message::DismissNotification(i))
        ]
        .align_y(Alignment::Center),
    )
    .padding(Padding::default().left(8.0).right(2.0).top(2.0).bottom(2.0))
    .style(style::notification_title(notification.severity));

    container(column![
        header,
        container(
            column![
                horizontal_rule(2.0).style(style::notification_timeout_indicator(
                    notification.severity,
                    notification.timeout
                )),
                scrollable(
                    container(text(&notification.content).size(14.0).width(Fill))
                        .padding(Padding::new(0.0).right(4.0))
                )
                .style(style::scrollable)
            ]
            .spacing(4.0)
        )
        .style(style::notification_content(notification.severity))
        .padding(Padding::new(4.0).top(0.0))
    ])
    .width(250.0)
    .max_height(300.0)
    .padding(Padding::new(2.0).top(1.0))
    .style(style::notification(notification.severity))
}

pub fn graph<'a, Message, Renderer, F, Data, Attachment>(
    data: &'a GraphData<Data, Attachment>,
    view_node: F,
) -> Graph<'a, Message, Renderer, Data, Attachment>
where
    Renderer: iced::advanced::image::Renderer + iced::advanced::graphics::geometry::Renderer,
    Data: std::fmt::Debug,
    Attachment: graph::Attachment + PartialEq + 'static,
    F: Fn(&GraphNode<Data>) -> Element<Message, iced::Theme, Renderer>,
{
    Graph::new(data, view_node)
}

pub fn dropdown<'a, Message: Clone + 'a>(
    dropdown_open: bool,
    show_hide_dropdown: Message,
    button_text: Text<'a, iced::Theme, iced::Renderer>,
    options: impl Iterator<Item = (char, impl Into<String>, Message)>,
) -> DropDown<'a, Message> {
    let underlay = button(
        row![
            button_text,
            if dropdown_open {
                icons::up()
            } else {
                icons::down()
            }
        ]
        .spacing(8.0),
    )
    .on_press(show_hide_dropdown.clone())
    .style(style::primary_button);

    let overlay = mouse_area(
        container(column(options.map(|(icon, option_text, on_press)| {
            button(
                row![text(icon).font(icons::ICON_FONT), text(option_text.into())]
                    .align_y(Alignment::Center)
                    .spacing(6.0),
            )
            .style(style::menu_button)
            .width(Fill)
            .on_press(on_press)
            .into()
        })))
        .padding(4.0)
        .style(style::dropdown),
    )
    .on_press(show_hide_dropdown.clone());

    DropDown::new(underlay, overlay, dropdown_open)
        .width(200.0)
        .on_dismiss(show_hide_dropdown)
}

#[allow(clippy::type_complexity)]
pub fn node<'a>(
    assets: &'a AssetsData,
) -> Box<dyn for<'any> Fn(&'any GraphNode<Node>) -> Element<'any, Message> + 'a> {
    Box::new(|node| match node.data() {
        Node::Character(handle) => {
            let Some(chara) = assets.get_direct::<Character>(*handle) else {
                return text("Couldn't load character.").into();
            };
            container(
                column![
                    image(
                        assets
                            .get_direct::<Image>(chara.img)
                            .map(|img| img.handle.clone())
                            .unwrap_or(default_image())
                    )
                    .width(Fill)
                    .height(Fill)
                    .filter_method(image::FilterMethod::Nearest),
                    opaque(
                        column![
                            text(chara.name.clone()).center().width(Fill),
                            base_button("ahkdlfjs").on_press(Message::CharacterButtonPressed)
                        ]
                        .width(Fill)
                        .spacing(5.0)
                    ),
                ]
                .spacing(5.0),
            )
            .width(150.0)
            .height(150.0)
            .padding(5.0)
            .style(style::node(node.selected()))
            .into()
        }
        Node::Family => container("")
            .width(10.0)
            .height(10.0)
            .style(|theme: &Theme| container::Style {
                text_color: None,
                background: Some(theme.palette().success.into()),
                border: Border::default().rounded(10.0),
                ..Default::default()
            })
            .into(),
    })
}
