use iced::{Point, Vector, widget::canvas::Path};

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
    fn connection_point(&self) -> Vector {
        Vector { x: 0.5, y: 0.5 }
    }

    fn path(_a: Self, a_point: Point, _b: Self, b_point: Point) -> Path {
        Path::line(a_point, b_point)
    }
}

#[derive(Debug, Clone)]
pub enum Edge {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Default, Debug, Clone)]
pub enum RelativeAttachment {
    #[default]
    Center,
    Edge {
        edge: Edge,
        align: f32,
    },
}

impl RelativeAttachment {
    pub fn all_edges<'a>() -> &'a [Self] {
        &[
            Self::Edge {
                edge: Edge::Top,
                align: 0.5,
            },
            Self::Edge {
                edge: Edge::Right,
                align: 0.5,
            },
            Self::Edge {
                edge: Edge::Bottom,
                align: 0.5,
            },
            Self::Edge {
                edge: Edge::Left,
                align: 0.5,
            },
        ]
    }

    pub fn all_corners<'a>() -> &'a [Self] {
        &[
            Self::Edge {
                edge: Edge::Top,
                align: 0.0,
            },
            Self::Edge {
                edge: Edge::Right,
                align: 0.0,
            },
            Self::Edge {
                edge: Edge::Bottom,
                align: 1.0,
            },
            Self::Edge {
                edge: Edge::Left,
                align: 1.0,
            },
        ]
    }
}

impl Attachment for RelativeAttachment {
    fn connection_point(&self) -> Vector {
        match self {
            Self::Center => Vector::new(0.5, 0.5),
            Self::Edge { edge, align } => match edge {
                Edge::Top => Vector::new(*align, 0.0),
                Edge::Right => Vector::new(1.0, *align),
                Edge::Bottom => Vector::new(*align, 1.0),
                Edge::Left => Vector::new(0.0, *align),
            },
        }
    }
}
