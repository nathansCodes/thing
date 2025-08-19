use iced::border::Radius;
use iced::gradient::{ColorStop, Linear};
use iced::theme::palette::Pair;
use iced::widget::{self, button, container, rule, slider};
use iced::{Border, Color, Gradient, Radians, Shadow, Theme, Vector};
use iced_aw::style::menu_bar;
use palette::convert::FromColorUnclamped;
use palette::rgb::Rgba;
use palette::{IntoColor, LinSrgba, Mix, Oklab};

use crate::notification::{self, Severity};

pub fn base_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let base = button::Style {
        border: Border::default().rounded(10.0),
        text_color: palette.background.base.text,
        ..button::primary(theme, status)
    };

    let btn_bg_a = palette.background.base.color;

    let btn_bg_b = if let button::Status::Disabled = status {
        palette.secondary.base.color
    } else {
        palette.primary.base.color
    };

    let btn_gradient_a = mix_colors(
        btn_bg_a,
        btn_bg_b,
        if let button::Status::Hovered | button::Status::Pressed = status {
            0.35
        } else {
            0.15
        },
    );

    let btn_gradient_b = mix_colors(
        btn_bg_a,
        btn_bg_b,
        if let button::Status::Hovered | button::Status::Pressed = status {
            0.6
        } else {
            0.4
        },
    );

    let angle = if status == button::Status::Pressed {
        Radians::PI
    } else {
        Radians::from(0.0)
    };

    let btn_bg = Gradient::Linear(Linear::new(angle).add_stops([
        ColorStop {
            offset: 0.0,
            color: btn_gradient_a,
        },
        ColorStop {
            offset: 0.6,
            color: btn_gradient_b,
        },
    ]));

    let btn_text = if let button::Status::Disabled = status {
        palette.secondary.base.color
    } else {
        palette.primary.base.color
    };

    button::Style {
        background: Some(btn_bg.into()),
        text_color: btn_text,
        border: Border {
            color: btn_gradient_b,
            width: 2.0,
            ..base.border
        },
        ..base
    }
}

pub fn scrollable(theme: &Theme, status: widget::scrollable::Status) -> widget::scrollable::Style {
    let default = widget::scrollable::default(theme, status);
    let palette = theme.extended_palette();

    let weak = mix_colors(
        palette.background.weak.color,
        palette.background.base.color,
        0.4,
    );

    let vrail_bg = Gradient::Linear(
        Linear::new(Radians::PI / 2.0)
            .add_stop(0.0, palette.background.base.color)
            .add_stop(0.2, weak)
            .add_stop(0.85, weak)
            .add_stop(1.0, palette.background.base.color),
    );

    widget::scrollable::Style {
        vertical_rail: widget::scrollable::Rail {
            background: Some(vrail_bg.into()),
            border: Border {
                color: palette.secondary.base.color,
                width: 1.0,
                radius: Radius::new(5.0),
            },
            scroller: widget::scrollable::Scroller {
                color: default.vertical_rail.scroller.color,
                border: Border {
                    color: mix_colors(
                        palette.background.base.color,
                        default.vertical_rail.scroller.color,
                        0.75,
                    ),
                    width: 1.0,
                    radius: Radius::new(5.0),
                },
            },
        },
        ..default
    }
}

pub fn menu_bar(theme: &Theme, _status: iced_aw::style::Status) -> menu_bar::Style {
    let palette = theme.extended_palette();

    let bar_gradient = Gradient::Linear(
        Linear::new(Radians::PI)
            .add_stop(
                0.0,
                mix_colors(
                    palette.background.base.color,
                    if palette.is_dark {
                        Color::BLACK
                    } else {
                        Color::WHITE
                    },
                    0.1,
                ),
            )
            .add_stop(0.7, palette.background.base.color),
    );

    // let highlight = mix_colors(
    //     palette.background.base.color,
    //     palette.background.weak.color,
    //     0.4,
    // );

    // BUG: shadow isn't shown if I use this gradient. Reintroduce gradient once
    // https://github.com/iced-rs/iced/issues/3036 is resolved
    // let menu_gradient = Gradient::Linear(
    //     Linear::new(Radians::PI)
    //         .add_stop(0.0, highlight)
    //         .add_stop(0.1, palette.background.base.color)
    //         .add_stop(0.9, palette.background.base.color)
    //         .add_stop(1.0, highlight),
    // );

    menu_bar::Style {
        bar_background: bar_gradient.into(),
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

pub fn menu_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let text = button::Style {
        text_color: palette.background.base.text,
        ..button::text(theme, status)
    };
    let base = base_button(theme, status);

    match status {
        button::Status::Active => text,
        button::Status::Disabled => button::Style {
            text_color: base.text_color.scale_alpha(0.8),
            ..text
        },
        _ => base,
    }
}

pub fn title_bar_active(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    let bg_start = mix_colors(
        palette.background.weak.color,
        palette.background.strong.color,
        0.5,
    );
    let bg_end = palette.background.strong.color;

    let gradient = Gradient::Linear(Linear::new(Radians::PI).add_stops([
        ColorStop {
            offset: 0.0,
            color: bg_start,
        },
        ColorStop {
            offset: 0.6,
            color: bg_end,
        },
    ]));

    container::Style {
        text_color: Some(palette.background.weak.text),
        background: Some(gradient.into()),
        border: Border {
            radius: Radius::new(8.0).bottom(0.0),
            width: 2.0,
            color: bg_end,
        },
        ..Default::default()
    }
}

pub fn title_bar_focused(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    let bg_start = mix_colors(
        palette.primary.weak.color,
        palette.primary.strong.color,
        0.3,
    );
    let bg_end = palette.primary.strong.color;

    let gradient = Gradient::Linear(Linear::new(Radians::PI).add_stops([
        ColorStop {
            offset: 0.0,
            color: bg_start,
        },
        ColorStop {
            offset: 0.6,
            color: bg_end,
        },
    ]));

    container::Style {
        text_color: Some(palette.primary.strong.text),
        background: Some(gradient.into()),
        border: Border {
            radius: Radius::new(8.0).bottom(0.0),
            width: 2.0,
            color: bg_end,
        },
        ..Default::default()
    }
}

pub fn pane_active(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    let bg_start = mix_colors(
        palette.background.base.color,
        palette.background.weak.color,
        0.75,
    );
    let bg_end = palette.background.weak.color;

    let gradient = Gradient::Linear(Linear::new(Radians::PI).add_stops([
        ColorStop {
            offset: 0.0,
            color: bg_start,
        },
        ColorStop {
            offset: 0.1,
            color: bg_end,
        },
    ]));

    container::Style {
        background: Some(gradient.into()),
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

    let bg_start = mix_colors(
        palette.primary.base.color,
        palette.background.weak.color,
        0.75,
    );
    let bg_end = mix_colors(
        palette.background.weak.color,
        palette.primary.base.color,
        0.1,
    );

    let gradient = Gradient::Linear(Linear::new(Radians::PI).add_stops([
        ColorStop {
            offset: 0.0,
            color: bg_start,
        },
        ColorStop {
            offset: 0.1,
            color: bg_end,
        },
    ]));

    container::Style {
        background: Some(gradient.into()),
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

pub fn node(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    let gradient_mid = mix_colors(
        palette.background.base.color,
        palette.background.weak.color,
        0.25,
    );

    let gradient_end = mix_colors(
        palette.background.base.color,
        palette.background.weak.color,
        0.6,
    );

    let bg = Gradient::Linear(
        Linear::new(Radians::PI)
            .add_stop(0.0, palette.background.base.color)
            .add_stop(0.1, gradient_mid)
            .add_stop(0.7, gradient_end),
    );

    container::Style::default().background(bg).border(
        Border::default()
            .rounded(15.0)
            .width(2.0)
            .color(gradient_end),
    )
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

fn notification_bg_color(theme: &Theme, severity: Severity) -> Pair {
    let palette = theme.extended_palette();

    match severity {
        Severity::Info => Pair {
            color: mix_colors(
                palette.primary.base.color,
                palette.background.weak.color,
                0.5,
            ),
            text: palette.background.base.text,
        },
        Severity::Destructive => Pair {
            color: mix_colors(
                palette.danger.base.color,
                palette.background.weak.color,
                0.5,
            ),
            text: palette.background.base.text,
        },
        Severity::Error => Pair {
            color: mix_colors(
                palette.background.weak.color,
                palette.danger.base.color,
                0.5,
            ),
            text: palette.background.base.text,
        },
    }
}

pub fn notification<'a>(severity: Severity) -> container::StyleFn<'a, Theme> {
    Box::new(move |theme: &Theme| {
        let Pair { color, text } = notification_bg_color(theme, severity);

        match severity {
            notification::Severity::Info => container::Style {
                text_color: Some(text),
                background: Some(color.scale_alpha(0.5).into()),
                shadow: Shadow {
                    color: Color::BLACK,
                    offset: Vector::ZERO,
                    blur_radius: 10.0,
                },
                border: Border::default().color(color).width(2.0).rounded(10.0),
            },
            notification::Severity::Destructive => container::Style {
                text_color: Some(text),
                background: Some(color.scale_alpha(0.5).into()),
                shadow: Shadow {
                    color: Color::BLACK,
                    offset: Vector::ZERO,
                    blur_radius: 10.0,
                },
                border: Border::default().color(color).width(2.0).rounded(10.0),
            },
            notification::Severity::Error => container::Style {
                text_color: Some(text),
                background: Some(color.scale_alpha(0.5).into()),
                shadow: Shadow {
                    color: Color::BLACK,
                    offset: Vector::ZERO,
                    blur_radius: 10.0,
                },
                border: Border::default().color(color).width(2.0).rounded(10.0),
            },
        }
    })
}

pub fn notification_title<'a>(severity: Severity) -> container::StyleFn<'a, Theme> {
    Box::new(move |theme: &Theme| {
        let Pair { color, text } = notification_bg_color(theme, severity);

        container::Style {
            text_color: Some(text),
            background: Some(color.into()),
            border: Border::default().rounded(Radius::default().top(10.0)),
            ..Default::default()
        }
    })
}

pub fn notification_close_button<'a>(severity: Severity) -> button::StyleFn<'a, Theme> {
    Box::new(move |theme: &Theme, status: button::Status| {
        let palette = theme.extended_palette();

        let base = button::Style {
            border: Border::default().rounded(10.0),
            text_color: palette.background.base.text,
            ..button::primary(theme, status)
        };

        let btn_bg_a = match severity {
            Severity::Info => palette.primary.base.color,
            Severity::Destructive => palette.danger.base.color,
            Severity::Error => palette.danger.base.color,
        };

        let btn_bg_b = palette.background.base.color;

        let btn_gradient_a = mix_colors(
            btn_bg_a,
            btn_bg_b,
            if let button::Status::Hovered | button::Status::Pressed = status {
                0.35
            } else {
                0.15
            },
        );

        let btn_gradient_b = mix_colors(
            btn_bg_a,
            btn_bg_b,
            if let button::Status::Hovered | button::Status::Pressed = status {
                0.6
            } else {
                0.4
            },
        );

        let angle = if status == button::Status::Pressed {
            Radians::PI
        } else {
            Radians::from(0.0)
        };

        let btn_bg = Gradient::Linear(Linear::new(angle).add_stops([
            ColorStop {
                offset: 0.0,
                color: btn_gradient_a,
            },
            ColorStop {
                offset: 0.6,
                color: btn_gradient_b,
            },
        ]));

        let btn_text = if let button::Status::Disabled = status {
            palette.secondary.base.color
        } else {
            btn_bg_a
        };

        button::Style {
            background: Some(btn_bg.into()),
            text_color: btn_text,
            border: Border {
                color: btn_gradient_b,
                width: 2.0,
                ..base.border
            },
            ..base
        }
    })
}

pub fn notification_timeout_indicator<'a>(
    severity: Severity,
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

    let weak_color = mix_colors(
        palette.primary.strong.color,
        palette.primary.weak.color,
        0.3,
    );
    let base_color = palette.primary.weak.color;

    let gradient = Gradient::Linear(Linear::new(Radians::PI).add_stops([
        ColorStop {
            offset: 0.0,
            color: weak_color,
        },
        ColorStop {
            offset: 1.0,
            color: base_color,
        },
    ]));

    let text_color = mix_colors(
        palette.primary.base.color,
        default
            .text_color
            .unwrap_or(if theme.extended_palette().is_dark {
                Color::BLACK
            } else {
                Color::WHITE
            }),
        0.8,
    );

    container::Style {
        background: Some(gradient.into()),
        border: Border::default().rounded(15.0).width(2.0).color(base_color),
        text_color: Some(text_color),
        ..default
    }
}

pub fn info_bar_border(theme: &Theme) -> container::Style {
    container::Style {
        border: Border {
            color: theme.extended_palette().background.weak.color,
            width: 2.0,
            radius: 17.0.into(),
        },
        shadow: Shadow {
            blur_radius: 10.0,
            color: Color::BLACK,
            offset: Vector::new(0.0, -2.0),
        },
        ..container::dark(theme)
    }
}

pub fn info_bar_zoom_slider(theme: &Theme, status: slider::Status) -> slider::Style {
    let palette = theme.extended_palette();

    let active_color_bright = palette.primary.strong.color;
    let active_color_dark = mix_colors(
        palette.primary.weak.color,
        palette.primary.strong.color,
        0.3,
    );

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

fn mix_colors(a: Color, b: Color, t: f32) -> Color {
    let a_srgba = LinSrgba::from(a.into_linear());
    let b_srgba = LinSrgba::from(b.into_linear());

    let a_oklab = Oklab::from_color_unclamped(a_srgba);
    let b_oklab = Oklab::from_color_unclamped(b_srgba);

    let mixed: Rgba = a_oklab.mix(b_oklab, t).into_color();
    mixed.into()
}
