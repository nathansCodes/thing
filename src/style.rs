use iced::border::Radius;
use iced::gradient::{ColorStop, Linear};
use iced::widget::{self, button, checkbox as iced_checkbox, container, rule, slider};
use iced::{Background, Border, Color, Gradient, Radians, Shadow, Theme, Vector};
use iced_aw::style::menu_bar;
use palette::convert::FromColorUnclamped;
use palette::rgb::Rgba;
use palette::{IntoColor, LinSrgba, Mix, Oklab};

use crate::notification::{self, Severity};

fn base_button(theme: &Theme, status: button::Status, accent: Color) -> button::Style {
    use button::Status::*;

    let palette = theme.extended_palette();

    let base = button::Style {
        border: Border::default().rounded(6.0),
        text_color: palette.background.base.text,
        ..button::primary(theme, status)
    };

    let btn_bg_a = theme.palette().background;

    let btn_bg_b = if let Disabled = status {
        mix_colors(accent, btn_bg_a, 0.3)
    } else {
        accent
    };

    let dark_t = match status {
        Active => 0.25,
        Hovered => 0.3,
        Pressed => 0.15,
        Disabled => 0.1,
    };

    let dark = mix_colors(btn_bg_a, btn_bg_b, dark_t);

    let darker_t = match status {
        Active => 0.225,
        Hovered => 0.275,
        Pressed => 0.125,
        Disabled => 0.075,
    };

    let darker = mix_colors(btn_bg_a, btn_bg_b, darker_t);

    let light_t = match status {
        Active => 0.5,
        Hovered => 0.5,
        Pressed => 0.45,
        Disabled => 0.4,
    };

    let light = mix_colors(btn_bg_a, btn_bg_b, light_t);

    let lighter = mix_colors(
        light,
        palette.background.base.text,
        if let Hovered = status { 0.25 } else { 0.075 },
    );

    let btn_bg = Gradient::Linear(Linear::new(Radians::from(0.0)).add_stops([
        ColorStop {
            offset: 0.0,
            color: light,
        },
        ColorStop {
            offset: 0.15,
            color: dark,
        },
        ColorStop {
            offset: 0.49,
            color: darker,
        },
        ColorStop {
            offset: 0.51,
            color: light,
        },
        ColorStop {
            offset: 0.625,
            color: light,
        },
        ColorStop {
            offset: 0.9,
            color: lighter,
        },
    ]));

    let border_color = mix_colors(lighter, dark, 0.3);

    button::Style {
        background: Some(btn_bg.into()),
        text_color: if let Disabled = status {
            mix_colors(accent, border_color, 0.5)
        } else {
            accent
        },
        border: Border {
            color: border_color,
            width: 2.0,
            ..base.border
        },
        ..base
    }
}

pub fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    base_button(theme, status, theme.extended_palette().primary.base.color)
}

pub fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
    base_button(theme, status, theme.palette().text)
}

pub fn success_button(theme: &Theme, status: button::Status) -> button::Style {
    base_button(theme, status, theme.extended_palette().success.base.color)
}

pub fn danger_button(theme: &Theme, status: button::Status) -> button::Style {
    base_button(theme, status, theme.extended_palette().danger.base.color)
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

pub fn text_input(theme: &Theme, status: widget::text_input::Status) -> widget::text_input::Style {
    use widget::text_input::*;

    let palette = theme.extended_palette();

    let border_color = match status {
        Status::Active => palette.background.strong.color,
        Status::Hovered => palette.primary.weak.color,
        Status::Focused => palette.primary.base.color,
        Status::Disabled => palette.secondary.base.color,
    };

    let bg_color = mix_colors(palette.background.base.color, border_color, 0.05);

    let bg = Gradient::Linear(
        Linear::new(Radians::PI)
            .add_stop(0.0, mix_colors(bg_color, Color::BLACK, 0.3))
            .add_stop(0.25, bg_color),
    );

    Style {
        background: bg.into(),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: Radius::new(8.0),
        },
        icon: border_color,
        placeholder: palette.secondary.weak.color,
        value: palette.background.base.text,
        selection: palette.primary.base.color,
    }
}

pub fn text_input_inline(
    theme: &Theme,
    status: widget::text_input::Status,
) -> widget::text_input::Style {
    use widget::text_input::*;

    let palette = theme.extended_palette();

    let border_color = match status {
        Status::Active => palette.background.strong.color,
        Status::Hovered => palette.primary.weak.color,
        Status::Focused => palette.primary.base.color,
        Status::Disabled => palette.secondary.base.color,
    };

    Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border::default(),
        icon: border_color,
        placeholder: palette.secondary.weak.color,
        value: palette.background.base.text,
        selection: palette.primary.base.color,
    }
}

pub fn search_bar(theme: &Theme, status: widget::text_input::Status) -> widget::text_input::Style {
    use widget::text_input::*;

    let palette = theme.extended_palette();

    let accent = match status {
        Status::Active => palette.background.strong.color,
        Status::Hovered => palette.primary.weak.color,
        Status::Focused => palette.primary.base.color,
        Status::Disabled => palette.secondary.base.color,
    };

    let bg_color = mix_colors(palette.background.base.color, accent, 0.05);

    let bg = Gradient::Linear(
        Linear::new(Radians::PI)
            .add_stop(0.0, mix_colors(bg_color, Color::BLACK, 0.3))
            .add_stop(0.25, bg_color),
    );

    Style {
        background: bg.into(),
        border: Border::default(),
        icon: accent,
        placeholder: palette.secondary.weak.color,
        value: palette.background.base.text,
        selection: palette.primary.base.color,
    }
}

pub fn checkbox(theme: &Theme, status: iced_checkbox::Status) -> iced_checkbox::Style {
    use iced_checkbox::*;

    let btn_style = match status {
        Status::Active { is_checked: true } => primary_button(theme, button::Status::Active),
        Status::Active { is_checked: false } => secondary_button(theme, button::Status::Active),
        Status::Hovered { is_checked: true } => primary_button(theme, button::Status::Hovered),
        Status::Hovered { is_checked: false } => secondary_button(theme, button::Status::Hovered),
        Status::Disabled { is_checked: true } => primary_button(theme, button::Status::Disabled),
        Status::Disabled { is_checked: false } => secondary_button(theme, button::Status::Disabled),
    };

    Style {
        background: btn_style.background.unwrap(),
        icon_color: btn_style.text_color,
        border: Border {
            radius: Radius::new(4),
            ..btn_style.border
        },
        text_color: None,
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
            .rounded(10.0)
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
    let base = primary_button(theme, status);

    match status {
        button::Status::Active => text,
        button::Status::Disabled => button::Style {
            text_color: base.text_color.scale_alpha(0.8),
            ..text
        },
        _ => base,
    }
}

pub fn dropdown(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.base.color.into()),
        border: Border::default()
            .width(2.0)
            .rounded(10.0)
            .color(palette.background.weak.color),
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::ZERO,
            blur_radius: 10.0,
        },
        ..Default::default()
    }
}

pub fn title_bar(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    let bg_start = mix_colors(
        palette.background.base.color,
        palette.background.weak.color,
        0.6,
    );
    let bg_mid = mix_colors(
        palette.background.base.color,
        palette.background.weak.color,
        0.8,
    );
    let bg_end = mix_colors(
        palette.background.base.color,
        palette.background.weak.color,
        0.7,
    );

    let bg = Gradient::Linear(Linear::new(0).add_stops([
        ColorStop {
            offset: 0.0,
            color: bg_start,
        },
        ColorStop {
            offset: 0.4,
            color: bg_mid,
        },
        ColorStop {
            offset: 0.85,
            color: bg_end,
        },
    ]));

    container::Style {
        background: Some(bg.into()),
        border: Border {
            color: Color::TRANSPARENT,
            width: 2.0,
            radius: Radius::new(0).top(8),
        },
        ..container::transparent(theme)
    }
}

pub fn title_bar_focused(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    let base = mix_colors(
        palette.background.base.color,
        palette.primary.base.color,
        0.1,
    );

    let weak = mix_colors(
        palette.background.weak.color,
        palette.primary.weak.color,
        0.15,
    );

    let bg_start = mix_colors(base, weak, 0.6);
    let bg_mid = mix_colors(base, weak, 0.8);
    let bg_end = mix_colors(base, weak, 0.7);

    let bg = Gradient::Linear(Linear::new(0).add_stops([
        ColorStop {
            offset: 0.0,
            color: bg_start,
        },
        ColorStop {
            offset: 0.4,
            color: bg_mid,
        },
        ColorStop {
            offset: 0.85,
            color: bg_end,
        },
    ]));

    container::Style {
        background: Some(bg.into()),
        border: Border {
            color: Color::TRANSPARENT,
            width: 2.0,
            radius: Radius::new(0).top(8),
        },
        ..container::transparent(theme)
    }
}

pub fn title_bar_label_container(theme: &Theme) -> container::Style {
    container::Style {
        shadow: Shadow {
            color: theme.palette().background.scale_alpha(0.5),
            offset: Vector::new(2.0, 3.0),
            blur_radius: 6.0,
        },
        border: Border {
            radius: Radius::new(0.0).top_left(6).bottom_right(6),
            ..Default::default()
        },
        ..container::transparent(theme)
    }
}

pub fn title_bar_label(theme: &Theme) -> container::Style {
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
            color: bg_end,
        },
        ColorStop {
            offset: 0.6,
            color: bg_start,
        },
    ]));

    let text_color = mix_colors(
        palette.background.base.text,
        palette.background.strong.text,
        0.5,
    );

    container::Style {
        text_color: Some(text_color),
        background: Some(gradient.into()),
        border: Border {
            radius: Radius::new(0).top_left(6).bottom_right(6),
            width: 2.0,
            color: bg_start,
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::ZERO,
            blur_radius: 4.0,
        },
    }
}

pub fn title_bar_label_focused(theme: &Theme) -> container::Style {
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
            radius: Radius::new(0).top_left(6).bottom_right(6),
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

pub fn node<'a>(selected: bool) -> container::StyleFn<'a, Theme> {
    Box::new(move |theme: &Theme| {
        let palette = theme.extended_palette();

        let base_color = if selected {
            palette.primary.base.color
        } else {
            palette.background.base.color
        };

        let gradient_mid = mix_colors(base_color, palette.background.weak.color, 0.25);

        let gradient_end = mix_colors(base_color, palette.background.weak.color, 0.6);

        let bg = Gradient::Linear(
            Linear::new(Radians::PI)
                .add_stop(0.0, base_color)
                .add_stop(0.1, gradient_mid)
                .add_stop(0.7, gradient_end),
        );

        container::Style {
            background: Some(bg.into()),
            border: Border::default()
                .rounded(10.0)
                .width(2.0)
                .color(gradient_end),
            text_color: selected.then_some(base_color),
            ..Default::default()
        }
    })
}

/// returns `(bg_gradient_start, bg_gradient_end, text_color)`
fn notification_bg_colors(theme: &Theme, severity: Severity) -> (Color, Color, Color) {
    let palette = theme.extended_palette();

    match severity {
        Severity::Info => (
            mix_colors(
                palette.primary.base.color,
                palette.background.weak.color,
                0.3,
            ),
            palette.primary.base.color,
            palette.background.base.text,
        ),
        Severity::Destructive => (
            mix_colors(
                palette.danger.base.color,
                palette.background.weak.color,
                0.3,
            ),
            palette.danger.base.color,
            palette.background.base.text,
        ),
        Severity::Error => (
            mix_colors(
                palette.danger.base.color,
                palette.background.weak.color,
                0.3,
            ),
            palette.danger.base.color,
            palette.background.base.text,
        ),
    }
}

pub fn notification<'a>(severity: Severity) -> container::StyleFn<'a, Theme> {
    Box::new(move |theme: &Theme| {
        let (_, bg_gradient_end, text) = notification_bg_colors(theme, severity);

        container::Style {
            text_color: Some(text),
            background: Some(Color::TRANSPARENT.into()),
            shadow: Shadow {
                color: Color::BLACK,
                offset: Vector::ZERO,
                blur_radius: 10.0,
            },
            border: Border::default()
                .color(bg_gradient_end)
                .width(2.0)
                .rounded(10.0),
        }
    })
}

pub fn notification_title<'a>(severity: Severity) -> container::StyleFn<'a, Theme> {
    Box::new(move |theme: &Theme| {
        let (bg_gradient_start, bg_gradient_end, _) = notification_bg_colors(theme, severity);
        let palette = theme.extended_palette();

        let bg = Gradient::Linear(
            Linear::new(Radians::PI)
                .add_stop(0.0, bg_gradient_start)
                .add_stop(0.5, bg_gradient_end),
        );

        container::Style {
            text_color: Some(palette.background.base.color),
            background: Some(bg.into()),
            border: Border::default().rounded(Radius::default().top(8.0)),
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

        let btn_bg_a = notification_bg_colors(theme, severity).1;

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

pub fn notification_content<'a>(severity: Severity) -> container::StyleFn<'a, Theme> {
    Box::new(move |theme: &Theme| {
        let (bg_gradient_start, bg_gradient_end, text) = notification_bg_colors(theme, severity);

        let bg = Gradient::Linear(
            Linear::new(Radians::PI)
                .add_stop(0.0, bg_gradient_end.scale_alpha(0.4))
                .add_stop(0.2, bg_gradient_start.scale_alpha(0.3))
                .add_stop(0.8, bg_gradient_start.scale_alpha(0.3))
                .add_stop(1.0, bg_gradient_end.scale_alpha(0.3)),
        );

        container::Style {
            text_color: Some(text),
            background: Some(bg.into()),
            border: Border::default().rounded(Radius::new(0.0).bottom(8.0)),
            ..Default::default()
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

pub fn dialog(theme: &Theme) -> container::Style {
    let default = container::dark(theme);
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            color: palette.background.strong.color,
            width: 2.0,
            radius: 20.0.into(),
        },
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::ZERO,
            blur_radius: 20.0,
        },
        ..default
    }
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

pub fn list_thumbnail(theme: &Theme, status: button::Status) -> button::Style {
    menu_button(theme, status)
}

pub fn list_item<'a>(even: bool) -> button::StyleFn<'a, Theme> {
    Box::new(move |theme: &Theme, status: button::Status| {
        let base = menu_button(theme, status);
        let grey = secondary_button(theme, button::Status::Hovered);

        let style = if even && let button::Status::Active | button::Status::Disabled = status {
            grey
        } else {
            base
        };

        button::Style {
            border: Border {
                radius: Radius::new(0),
                ..style.border
            },
            ..style
        }
    })
}

fn mix_colors(a: Color, b: Color, t: f32) -> Color {
    let a_srgba = LinSrgba::from(a.into_linear());
    let b_srgba = LinSrgba::from(b.into_linear());

    let a_oklab = Oklab::from_color_unclamped(a_srgba);
    let b_oklab = Oklab::from_color_unclamped(b_srgba);

    let mixed: Rgba = a_oklab.mix(b_oklab, t).into_color();
    mixed.into()
}
