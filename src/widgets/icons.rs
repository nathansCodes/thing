use iced::{Element, Font, advanced::widget::Text, widget::text};

pub const ICON_FONT: Font = Font::with_name("things");

fn icon<'a>(codepoint: char) -> Text<'a, iced::Theme, iced::Renderer> {
    text(codepoint)
        .font(ICON_FONT)
        .shaping(text::Shaping::Advanced)
}

pub const SEARCH: char = '\u{E800}';

pub fn search<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(SEARCH)
}

pub const PAUSE: char = '\u{E801}';

pub fn pause<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(PAUSE)
}

pub const STOP: char = '\u{E802}';

pub fn stop<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(STOP)
}

pub const PLAY: char = '\u{E803}';

pub fn play<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(PLAY)
}

pub const THUMBNAILS: char = '\u{E804}';

pub fn thumbnails<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(THUMBNAILS)
}

pub const LIST: char = '\u{E805}';

pub fn list<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(LIST)
}

pub const DOWN: char = '\u{E806}';

pub fn down<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(DOWN)
}

pub const UP: char = '\u{E807}';

pub fn up<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(UP)
}

pub const IMAGE: char = '\u{E808}';

pub fn image<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(IMAGE)
}

pub const USER: char = '\u{E809}';

pub fn user<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(USER)
}

pub const EDIT: char = '\u{E80A}';

pub fn edit<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(EDIT)
}

pub const RENAME: char = '\u{E80B}';

pub fn rename<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(RENAME)
}

pub const UNDO: char = '\u{E80C}';

pub fn undo<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(UNDO)
}

pub const REDO: char = '\u{E80D}';

pub fn redo<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(REDO)
}

pub const CLOSE: char = '\u{E80E}';

pub fn close<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(CLOSE)
}

pub const CANCEL_FILLED: char = '\u{E80F}';

pub fn cancel_filled<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(CANCEL_FILLED)
}

pub const CANCEL: char = '\u{E810}';

pub fn cancel<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(CANCEL)
}

pub const CHECK: char = '\u{E811}';

pub fn check<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(CHECK)
}

pub const PIN: char = '\u{E812}';

pub fn pin<'a>() -> Text<'a, iced::Theme, iced::Renderer> {
    icon(PIN)
}
