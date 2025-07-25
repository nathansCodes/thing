use iced::border::Radius;
use iced::widget::container;
use iced::{Border, Color, Shadow, Theme, Vector};

pub fn menu_bar(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.weak.color.into()),
        ..Default::default()
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
