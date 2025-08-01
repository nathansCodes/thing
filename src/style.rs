use iced::border::Radius;
use iced::widget::{button, container};
use iced::{Border, Color, Shadow, Theme, Vector};
use iced_aw::style::menu_bar;

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
