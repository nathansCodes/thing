use iced::{
    Element, Font,
    Length::{Fill, Shrink},
    Theme,
    font::Weight,
    widget::{
        button, column, container, horizontal_space, mouse_area, opaque, row, stack, text,
        vertical_space,
    },
};

use crate::style;

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Success,
    Neutral,
    Warn,
    Destructive,
}

#[derive(Debug, Clone)]
pub struct DialogOption<Message> {
    severity: Severity,
    text: String,
    on_press: Message,
}

impl<Message> DialogOption<Message> {
    pub fn new(severity: Severity, text: impl Into<String>, on_press: Message) -> Self {
        Self {
            severity,
            text: text.into(),
            on_press,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dialog<Message> {
    title: String,
    body_text: String,
    on_cancel: Message,
    options: Vec<DialogOption<Message>>,
}

impl<Message> Dialog<Message> {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        on_cancel: Message,
        options: Vec<DialogOption<Message>>,
    ) -> Self {
        Self {
            title: title.into(),
            body_text: body.into(),
            on_cancel,
            options,
        }
    }
}

pub fn dialog<'a, Message: Clone>(
    dialog: &'a Option<Dialog<Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut title_font = Font::DEFAULT;
    title_font.weight = Weight::Bold;

    stack!(content.into())
        .push_maybe(dialog.clone().map(|dialog| {
            opaque(
                mouse_area(
                    container(opaque(
                        container(
                            column![
                                text(dialog.title).font(title_font).size(24.0),
                                text(dialog.body_text).style(|theme: &Theme| {
                                    text::Style {
                                        color: Some(theme.palette().text),
                                    }
                                }),
                                vertical_space(),
                                row![
                                    button(text("Cancel").center())
                                        .on_press(dialog.on_cancel.clone())
                                        .height(40.0)
                                        .style(style::secondary_button),
                                    horizontal_space(),
                                ]
                                .extend(dialog.options.iter().cloned().map(|opt| {
                                    button(text(opt.text).center())
                                        .on_press(opt.on_press)
                                        .height(40.0)
                                        .style(match opt.severity {
                                            Severity::Success => style::success_button,
                                            Severity::Neutral => style::primary_button,
                                            Severity::Warn => style::danger_button,
                                            Severity::Destructive => style::danger_button,
                                        })
                                        .into()
                                }))
                                .spacing(10.0)
                                .width(Fill)
                            ]
                            .height(Fill)
                            .spacing(4.0),
                        )
                        .style(style::dialog)
                        .center_x(Shrink)
                        .center_y(Shrink)
                        .max_width(400.0)
                        .max_height(300.0)
                        .padding(16.0),
                    ))
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();
                        container::Style {
                            background: Some(palette.background.base.color.scale_alpha(0.4).into()),
                            ..container::transparent(theme)
                        }
                    })
                    .center_x(Fill)
                    .center_y(Fill),
                )
                .on_press(dialog.on_cancel),
            )
        }))
        .into()
}
