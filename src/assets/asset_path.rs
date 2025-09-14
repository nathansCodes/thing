use std::{
    fmt::Display,
    ops::{Add, AddAssign},
    path::PathBuf,
    str::FromStr,
};

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
        self.name.split('.').next().unwrap_or("")
    }

    pub fn file_name(&self) -> &str {
        &self.name
    }

    pub fn extension(&self) -> &str {
        let first_dot = self.name.find('.');

        first_dot.map_or("", |pos| &self.name[pos..])
    }
}

impl TryFrom<&str> for AssetPath {
    type Error = ();

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        let mut components = name.split('/');

        let kind = AssetKind::from_str(components.next().ok_or(())?)?;

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

impl AddAssign<AssetPath> for PathBuf {
    fn add_assign(&mut self, rhs: AssetPath) {
        self.push(format!("{}", rhs))
    }
}

impl Add<&AssetPath> for &PathBuf {
    type Output = PathBuf;

    fn add(self, rhs: &AssetPath) -> Self::Output {
        self.join(format!("{}", rhs))
    }
}

impl Add<AssetPath> for &PathBuf {
    type Output = PathBuf;

    fn add(self, rhs: AssetPath) -> Self::Output {
        self.join(format!("{}", rhs))
    }
}

impl Add<AssetPath> for PathBuf {
    type Output = PathBuf;

    fn add(self, rhs: AssetPath) -> Self::Output {
        self.join(format!("{}", rhs))
    }
}

impl Add<&str> for AssetKind {
    type Output = AssetPath;

    fn add(self, name: &str) -> Self::Output {
        AssetPath {
            kind: self,
            name: name.to_string(),
        }
    }
}

impl Add<String> for AssetKind {
    type Output = AssetPath;

    fn add(self, name: String) -> Self::Output {
        AssetPath { kind: self, name }
    }
}
