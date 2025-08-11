use std::marker::PhantomData;

use iced::{Point, Size, Vector, widget::canvas::Path};
use serde::{Deserialize, Serialize};

use crate::graph::line_styles;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct Connection<A: Attachment = RelativeAttachment> {
    pub a: (usize, A),
    pub b: (usize, A),
}

impl<A: Attachment> Connection<A> {
    pub(super) fn new(a: usize, a_attachment: A, b: usize, b_attachment: A) -> Self {
        Self {
            a: (a, a_attachment),
            b: (b, b_attachment),
        }
    }
}

pub trait Attachment: std::fmt::Debug + Clone + Send {
    fn connection_point(&self) -> Vector;

    fn path(_a: Self, a_point: Point, _b: Self, b_point: Point) -> Path {
        Path::line(a_point, b_point)
    }

    fn resolve(&self, size: Size, position: Point) -> Point {
        position + (size * self.connection_point()).into()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Edge {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelativeAttachment<Style = line_styles::Direct>
where
    Style: line_styles::LineStyle + std::fmt::Debug + Clone,
{
    #[default]
    Center,
    Edge {
        edge: Edge,
        align: f32,
        #[serde(skip)]
        _phantom: PhantomData<Style>,
    },
}

impl<Style> RelativeAttachment<Style>
where
    Style: line_styles::LineStyle + std::fmt::Debug + Clone,
{
    pub fn all_edges(size: Vector) -> [(Self, Vector); 4] {
        [
            (
                Self::Edge {
                    edge: Edge::Top,
                    align: 0.5,
                    _phantom: PhantomData,
                },
                size,
            ),
            (
                Self::Edge {
                    edge: Edge::Right,
                    align: 0.5,
                    _phantom: PhantomData,
                },
                size,
            ),
            (
                Self::Edge {
                    edge: Edge::Bottom,
                    align: 0.5,
                    _phantom: PhantomData,
                },
                size,
            ),
            (
                Self::Edge {
                    edge: Edge::Left,
                    align: 0.5,
                    _phantom: PhantomData,
                },
                size,
            ),
        ]
    }

    pub fn top() -> Self {
        Self::Edge {
            edge: Edge::Top,
            align: 0.5,
            _phantom: PhantomData,
        }
    }

    pub fn right() -> Self {
        Self::Edge {
            edge: Edge::Right,
            align: 0.5,
            _phantom: PhantomData,
        }
    }

    pub fn bottom() -> Self {
        Self::Edge {
            edge: Edge::Bottom,
            align: 0.5,
            _phantom: PhantomData,
        }
    }

    pub fn left() -> Self {
        Self::Edge {
            edge: Edge::Left,
            align: 0.5,
            _phantom: PhantomData,
        }
    }

    pub fn is_top(&self) -> bool {
        matches!(
            self,
            Self::Edge {
                edge: Edge::Top,
                ..
            }
        )
    }

    pub fn is_right(&self) -> bool {
        matches!(
            self,
            Self::Edge {
                edge: Edge::Right,
                ..
            }
        )
    }

    pub fn is_bottom(&self) -> bool {
        matches!(
            self,
            Self::Edge {
                edge: Edge::Bottom,
                ..
            }
        )
    }

    pub fn is_left(&self) -> bool {
        matches!(
            self,
            Self::Edge {
                edge: Edge::Left,
                ..
            }
        )
    }

    pub fn is_center(&self) -> bool {
        matches!(self, Self::Center)
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(
            self,
            Self::Edge {
                edge: Edge::Left | Edge::Right,
                ..
            }
        )
    }

    pub fn is_vertical(&self) -> bool {
        matches!(
            self,
            Self::Edge {
                edge: Edge::Top | Edge::Bottom,
                ..
            }
        )
    }
}

impl<Style> TryFrom<RelativeAttachment<Style>> for Edge
where
    Style: line_styles::LineStyle,
{
    type Error = ();

    fn try_from(value: RelativeAttachment<Style>) -> Result<Self, Self::Error> {
        match value {
            RelativeAttachment::Center => Err(()),
            RelativeAttachment::Edge { edge, .. } => Ok(edge),
        }
    }
}

impl<Style> Attachment for RelativeAttachment<Style>
where
    Style: line_styles::LineStyle + std::fmt::Debug + Clone + Send,
{
    fn connection_point(&self) -> Vector {
        match self {
            Self::Center => Vector::new(0.5, 0.5),
            Self::Edge { edge, align, .. } => match edge {
                Edge::Top => Vector::new(*align, 0.0),
                Edge::Right => Vector::new(1.0, *align),
                Edge::Bottom => Vector::new(*align, 1.0),
                Edge::Left => Vector::new(0.0, *align),
            },
        }
    }

    fn path(a: Self, a_point: Point, b: Self, b_point: Point) -> Path {
        Style::path(a, a_point, b, b_point)
    }
}
