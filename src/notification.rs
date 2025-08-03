#[derive(Debug, Clone)]
pub struct Notification {
    pub severity: Severity,
    pub title: String,
    pub content: String,
    pub timeout: f32,
}

impl Notification {
    pub fn info(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            severity: Severity::Info,
            title: title.into(),
            content: content.into(),
            timeout: 5.0,
        }
    }

    pub fn destructive(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            severity: Severity::Destructive,
            title: title.into(),
            content: content.into(),
            timeout: 5.0,
        }
    }

    pub fn error(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            title: title.into(),
            content: content.into(),
            timeout: 5.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Severity {
    Info,
    Destructive,
    Error,
}
