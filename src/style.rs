use iced::border::Radius;
use iced::gradient::{ColorStop, Linear};
use iced::widget::slider::Handle;
use iced::widget::{button, container, rule, slider};
use iced::{Border, Color, Gradient, Radians, Shadow, Theme, Vector};
use iced_aw::style::menu_bar;
use palette::convert::FromColorUnclamped;
use palette::{IntoColor, Mix, Oklab, Srgba};

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

pub fn info_bar(theme: &Theme) -> container::Style {
    let default = container::dark(theme);
    let palette = theme.extended_palette();

    let base_color = palette.primary.weak.color;

    let gradient = Gradient::Linear(Linear::new(Radians::PI).add_stops([
        ColorStop {
            offset: 0.0,
            color: base_color.scale_alpha(0.5),
        },
        ColorStop {
            offset: 1.0,
            color: base_color,
        },
    ]));

    let accent_strong = Srgba::from(palette.primary.strong.color.into_linear());
    let default_text_color = Srgba::from(default.text_color.unwrap_or(Color::WHITE).into_linear());

    let accent_strong = Oklab::from_color_unclamped(accent_strong);
    let default_text_color = Oklab::from_color_unclamped(default_text_color);

    let text_color: Srgba = accent_strong.mix(default_text_color, 0.8).into_color();

    container::Style {
        background: Some(gradient.into()),
        border: Border::default().rounded(10.0).width(1.0).color(base_color),
        text_color: Some(Color::from(text_color)),
        shadow: Shadow {
            blur_radius: 10.0,
            color: Color::from(text_color),
            ..Default::default()
        },
    }
}

pub fn info_bar_zoom_slider(theme: &Theme, status: slider::Status) -> slider::Style {
    let palette = theme.extended_palette();

    let active_color_bright = palette.primary.strong.color;
    let active_color_dark = palette.primary.weak.color;

    let active_linear_gradient = Linear::new(Radians::PI).add_stops([
        ColorStop {
            offset: 0.0,
            color: active_color_bright,
        },
        ColorStop {
            offset: 1.0,
            color: active_color_dark,
        },
    ]);

    let active_gradient = Gradient::Linear(active_linear_gradient);

    let muted_color = palette.secondary.base.color;

    let muted_gradient = Gradient::Linear(Linear::new(Radians::PI).add_stops([
        ColorStop {
            offset: 0.0,
            color: muted_color.scale_alpha(0.5),
        },
        ColorStop {
            offset: 1.0,
            color: muted_color,
        },
    ]));

    let handle_gradient = match status {
        slider::Status::Active => active_gradient,
        slider::Status::Hovered => active_gradient.scale_alpha(0.8),
        slider::Status::Dragged => Gradient::Linear(
            Linear::new(0.0).add_stops(active_linear_gradient.stops.iter().filter_map(|s| *s)),
        ),
    };

    slider::Style {
        rail: slider::Rail {
            backgrounds: (active_gradient.into(), muted_gradient.into()),
            border: Border::default()
                .color(active_color_bright)
                .width(1.0)
                .rounded(3.0),
            width: 6.0,
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 7.0 },
            background: handle_gradient.into(),
            border_width: 1.0,
            border_color: active_color_bright,
        },
    }
}
