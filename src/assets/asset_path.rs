use std::{fmt::Display, ops::Add, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::assets::AssetKind;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AssetPath {
    kind: AssetKind,
    name: String,
}

impl AssetPath {
    pub fn new(kind: AssetKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
        }
    }

    pub fn kind(&self) -> AssetKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<&str> for AssetPath {
    type Error = ();

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        let mut components = name.split('/');

        let kind = AssetKind::try_from(components.next().ok_or(())?)?;

        Ok(AssetPath {
            kind,
            name: components.next().ok_or(())?.to_string(),
        })
    }
}

impl Display for AssetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}/{}", self.kind.folder(), self.name))
    }
}

impl Add<AssetPath> for PathBuf {
    type Output = PathBuf;

    fn add(self, rhs: AssetPath) -> Self::Output {
        self.join(format!("{}", rhs))
    }
}

impl Add<String> for AssetKind {
    type Output = AssetPath;

    fn add(self, name: String) -> Self::Output {
        AssetPath { kind: self, name }
    }
}
