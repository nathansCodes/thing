use iced::border::Radius;
use iced::widget::{button, container, rule};
use iced::{Border, Color, Shadow, Theme, Vector};
use iced_aw::style::menu_bar;

use crate::notification::{self, Severity};

pub fn base_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let base = button::Style {
        border: Border::default().rounded(10.0),
        text_color: palette.background.base.text,
        ..button::primary(theme, status)
    };

    match status {
        button::Status::Active => button::Style {
            background: Some(palette.primary.base.color.scale_alpha(0.15).into()),
            text_color: palette.primary.base.color,
            ..base
        },
        button::Status::Hovered => button::Style {
            background: Some(palette.primary.base.color.into()),
            text_color: palette.primary.base.text,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(palette.primary.base.color.scale_alpha(0.15).into()),
            text_color: palette.primary.base.color,
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(palette.secondary.base.color.scale_alpha(0.15).into()),
            text_color: palette.secondary.base.color,
            ..base
        },
    }
}

pub fn menu_bar(theme: &Theme, _status: iced_aw::style::Status) -> menu_bar::Style {
    let palette = theme.extended_palette();

    menu_bar::Style {
        bar_background: palette.background.base.color.into(),
        menu_background: palette.background.base.color.into(),
        menu_border: Border::default()
            .width(2.0)
            .rounded(15.0)
            .color(palette.background.weak.color),
        menu_shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::ZERO,
            blur_radius: 10.0,
        },
        ..Default::default()
    }
}

pub fn menu_item(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let base = button::Style {
        border: Border::default().rounded(10.0),
        text_color: palette.background.base.text,
        ..button::text(theme, status)
    };

    match status {
        button::Status::Active => base,
        button::Status::Pressed => button::Style {
            background: Some(palette.primary.base.color.into()),
            text_color: palette.primary.base.text,
            ..base
        },
        button::Status::Hovered => button::Style {
            background: Some(palette.primary.base.color.scale_alpha(0.15).into()),
            text_color: palette.primary.base.color,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: base.text_color.scale_alpha(0.8),
            ..base
        },
    }
}

pub fn title_bar_active(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.background.strong.text),
        background: Some(palette.background.strong.color.into()),
        border: Border {
            radius: Radius::new(8.0).bottom(0.0),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn title_bar_focused(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        text_color: Some(palette.primary.strong.text),
        background: Some(palette.primary.strong.color.into()),
        border: Border {
            radius: Radius::new(8.0).bottom(0.0),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn pane_active(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            width: 2.0,
            color: palette.background.strong.color,
            radius: Radius::new(8.0),
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::ZERO,
            blur_radius: 5.0,
        },
        ..Default::default()
    }
}

pub fn pane_focused(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            width: 2.0,
            color: palette.primary.strong.color,
            radius: Radius::new(8.0),
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::ZERO,
            blur_radius: 10.0,
        },
        ..Default::default()
    }
}

pub fn graph_overlay(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.base.color.into()),
        border: Border {
            width: 2.0,
            color: palette.primary.weak.color,
            radius: Radius::new(8.0),
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::ZERO,
            blur_radius: 10.0,
        },
        ..Default::default()
    }
}

pub fn notification<'a>(severity: &'a Severity) -> container::StyleFn<'a, Theme> {
    Box::new(move |theme: &Theme| {
        let palette = theme.extended_palette();

        match severity {
            notification::Severity::Info => container::Style {
                text_color: Some(palette.primary.base.color),
                background: Some(palette.primary.base.color.scale_alpha(0.5).into()),
                shadow: Shadow {
                    color: Color::BLACK,
                    offset: Vector::ZERO,
                    blur_radius: 10.0,
                },
                border: Border::default()
                    .color(palette.primary.base.color)
                    .width(2.0)
                    .rounded(10.0),
            },
            notification::Severity::Destructive => container::Style {
                text_color: Some(palette.danger.base.color),
                background: Some(palette.danger.base.color.scale_alpha(0.5).into()),
                shadow: Shadow {
                    color: Color::BLACK,
                    offset: Vector::ZERO,
                    blur_radius: 10.0,
                },
                border: Border::default()
                    .color(palette.danger.base.color)
                    .width(2.0)
                    .rounded(10.0),
            },
            notification::Severity::Error => container::Style {
                text_color: Some(palette.danger.base.color),
                background: Some(palette.danger.base.color.scale_alpha(0.5).into()),
                shadow: Shadow {
                    color: Color::BLACK,
                    offset: Vector::ZERO,
                    blur_radius: 10.0,
                },
                border: Border::default()
                    .color(palette.danger.base.color)
                    .width(2.0)
                    .rounded(10.0),
            },
        }
    })
}

pub fn notification_title<'a>(severity: &'a Severity) -> container::StyleFn<'a, Theme> {
    Box::new(move |theme: &Theme| {
        let palette = theme.extended_palette();

        match severity {
            notification::Severity::Info => container::Style {
                text_color: Some(palette.background.base.color),
                background: Some(palette.primary.base.color.into()),
                border: Border::default().rounded(Radius::default().top(10.0)),
                ..Default::default()
            },
            notification::Severity::Destructive => container::Style {
                text_color: Some(palette.background.base.color),
                background: Some(palette.danger.base.color.into()),
                border: Border::default().rounded(Radius::default().top(10.0)),
                ..Default::default()
            },
            notification::Severity::Error => container::Style {
                text_color: Some(palette.background.base.color),
                background: Some(palette.danger.base.color.into()),
                border: Border::default().rounded(Radius::default().top(10.0)),
                ..Default::default()
            },
        }
    })
}

pub fn notification_close_button<'a>(severity: &'a Severity) -> button::StyleFn<'a, Theme> {
    Box::new(move |theme: &Theme, status: button::Status| {
        let palette = theme.extended_palette();

        let base_color = match severity {
            notification::Severity::Info => palette.primary.base.color,
            notification::Severity::Destructive => palette.danger.base.color,
            notification::Severity::Error => palette.danger.base.color,
        };

        match status {
            button::Status::Active | button::Status::Pressed => button::Style {
                text_color: palette.background.base.color,
                background: Some(base_color.into()),
                border: Border::default().rounded(100.0),
                ..Default::default()
            },
            button::Status::Hovered => button::Style {
                text_color: base_color,
                background: Some(palette.background.base.color.into()),
                border: Border::default().rounded(100.0),
                ..Default::default()
            },
            button::Status::Disabled => unimplemented!(),
        }
    })
}

pub fn notification_timeout_indicator<'a>(
    severity: &'a Severity,
    timeout: f32,
) -> rule::StyleFn<'a, Theme> {
    Box::new(move |theme: &Theme| {
        let palette = theme.extended_palette();

        let color = match severity {
            notification::Severity::Info => palette.primary.base.color,
            notification::Severity::Destructive => palette.danger.base.color,
            notification::Severity::Error => palette.danger.base.color,
        };

        rule::Style {
            color,
            width: 2,
            radius: Radius::new(1.0),
            fill_mode: rule::FillMode::Percent(timeout / 5.0 * 100.0),
        }
    })
}
