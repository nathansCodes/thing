use iced::{Element, Font, advanced::widget::Text, widget::text};

pub const ICON_FONT: Font = Font::with_name("things");

fn icon<'a>(codepoint: char) -> Text<'a, iced::Theme, iced::Renderer> {
    text(codepoint).font(ICON_FONT)
}

pub const CLOSE: char = '\u{E801}';

pub fn close<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(CLOSE)
}

pub const SEARCH: char = '\u{E800}';

pub fn search<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(SEARCH)
}

pub const PAUSE: char = '\u{E803}';

pub fn pause<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(PAUSE)
}

pub const STOP: char = '\u{E804}';

pub fn stop<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(STOP)
}

pub const PLAY: char = '\u{E805}';

pub fn play<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(PLAY)
}

pub const THUMBNAILS: char = '\u{E802}';

pub fn thumbnails<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(THUMBNAILS)
}

pub const LIST: char = '\u{E806}';

pub fn list<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(LIST)
}
