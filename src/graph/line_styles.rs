use iced::{Point, Vector, widget::canvas::Path};

use crate::graph::{RelativeAttachment, connections::Edge};

pub trait LineStyle: Sized + std::fmt::Debug + Clone {
    fn path(
        _a: RelativeAttachment<Self>,
        a_point: Point,
        _b: RelativeAttachment<Self>,
        b_point: Point,
    ) -> Path;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Direct;

impl LineStyle for Direct {
    fn path(
        _a: RelativeAttachment<Self>,
        a_point: Point,
        _b: RelativeAttachment<Self>,
        b_point: Point,
    ) -> Path {
        Path::line(a_point, b_point)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AxisAligned;

impl LineStyle for AxisAligned {
    fn path(
        a: RelativeAttachment<Self>,
        a_point: Point,
        b: RelativeAttachment<Self>,
        b_point: Point,
    ) -> Path {
        let a_edge = match a {
            RelativeAttachment::Edge { edge: a_edge, .. } => a_edge,
            RelativeAttachment::Center => {
                let diff = b_point - a_point;

                if diff.y > diff.x && diff.y < -diff.x {
                    Edge::Left
                } else if diff.y < diff.x && diff.y < -diff.x {
                    Edge::Top
                } else if diff.y < diff.x && diff.y > -diff.x {
                    Edge::Right
                } else {
                    Edge::Bottom
                }
            }
        };

        let b_edge = match b {
            RelativeAttachment::Edge { edge: b_edge, .. } => b_edge,
            RelativeAttachment::Center => {
                let diff = a_point - b_point;

                if diff.y > diff.x && diff.y < -diff.x {
                    Edge::Left
                } else if diff.y < diff.x && diff.y < -diff.x {
                    Edge::Top
                } else if diff.y < diff.x && diff.y > -diff.x {
                    Edge::Right
                } else {
                    Edge::Bottom
                }
            }
        };

        let mut a_point = a_point;
        let mut b_point = b_point;

        // swap if needed
        let (a, b) = match (a_edge, b_edge) {
            (Edge::Left, Edge::Right) => {
                std::mem::swap(&mut a_point, &mut b_point);

                (Edge::Right, Edge::Left)
            }
            (Edge::Bottom, Edge::Top) => {
                std::mem::swap(&mut a_point, &mut b_point);

                (Edge::Top, Edge::Bottom)
            }
            (a, b)
                if (a == Edge::Top || a == Edge::Bottom)
                    && (b == Edge::Left || b == Edge::Right) =>
            {
                std::mem::swap(&mut a_point, &mut b_point);

                (b, a.clone())
            }
            others => others,
        };

        let a_direction = match a {
            Edge::Top => Vector::new(0.0, -1.0),
            Edge::Bottom => Vector::new(0.0, 1.0),
            Edge::Left => Vector::new(-1.0, 0.0),
            Edge::Right => Vector::new(1.0, 0.0),
        };

        let b_direction = match b {
            Edge::Top => Vector::new(0.0, -1.0),
            Edge::Bottom => Vector::new(0.0, 1.0),
            Edge::Left => Vector::new(-1.0, 0.0),
            Edge::Right => Vector::new(1.0, 0.0),
        };

        Path::new(|builder| {
            builder.move_to(a_point);

            let a_vector = Vector::new(a_point.x, a_point.y);
            let b_vector = Vector::new(b_point.x, b_point.y);

            let a_stub = a_point + a_direction * 25.0;
            let b_stub = b_point + b_direction * 25.0;

            match (a, b) {
                // if they're the same
                (a, b) if a == b => {
                    let (connecting_start, connecting_end) = match a_direction {
                        Vector { x: 0.0, y } => {
                            let mut connecting_start = a_stub;
                            let mut connecting_end = b_stub;

                            let y = if y == 1.0 {
                                f32::max(connecting_start.y, connecting_end.y)
                            } else {
                                f32::min(connecting_start.y, connecting_end.y)
                            };

                            connecting_start.y = y;
                            connecting_end.y = y;

                            (connecting_start, connecting_end)
                        }
                        Vector { x, y: 0.0 } => {
                            let mut connecting_start = a_stub;
                            let mut connecting_end = b_stub;

                            let x = if x == 1.0 {
                                f32::max(connecting_start.x, connecting_end.x)
                            } else {
                                f32::min(connecting_start.x, connecting_end.x)
                            };

                            connecting_start.x = x;
                            connecting_end.x = x;

                            (connecting_start, connecting_end)
                        }
                        _ => unreachable!(),
                    };

                    builder.line_to(connecting_start);
                    builder.line_to(connecting_end);

                    builder.line_to(b_point);
                }
                // side and other side
                // or top and bottom
                (Edge::Top, Edge::Bottom) | (Edge::Right, Edge::Left) => {
                    let halfway_vector = (b_vector - a_vector) * 0.5;

                    builder.line_to(a_stub);

                    if a_stub.x * a_direction.x.abs() < b_stub.x * b_direction.x.abs()
                        || a_stub.y * a_direction.y.abs() > b_stub.y * b_direction.y.abs()
                    {
                        builder.line_to(Point::new(
                            a_point.x + halfway_vector.x * a_direction.x,
                            a_point.y - halfway_vector.y * a_direction.y,
                        ));

                        builder.line_to(Point::new(
                            b_point.x + halfway_vector.x * b_direction.x,
                            b_point.y - halfway_vector.y * b_direction.y,
                        ));
                    } else if a_direction.x == 0.0 {
                        builder.line_to(Point::new(a_stub.x + halfway_vector.x, a_stub.y));
                        builder.line_to(Point::new(b_stub.x - halfway_vector.x, b_stub.y));
                    } else {
                        builder.line_to(Point::new(a_stub.x, a_stub.y + halfway_vector.y));
                        builder.line_to(Point::new(b_stub.x, b_stub.y - halfway_vector.y));
                    }

                    builder.line_to(b_stub);

                    builder.line_to(b_point);
                }
                // side with top/bottom
                (Edge::Left, Edge::Top)
                | (Edge::Right, Edge::Top)
                | (Edge::Left, Edge::Bottom)
                | (Edge::Right, Edge::Bottom) => {
                    builder.line_to(a_stub);

                    builder.line_to(Point::new(a_stub.x, b_stub.y));

                    builder.line_to(b_stub);
                    builder.line_to(b_point);
                }
                _ => unreachable!(),
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bezier;

impl LineStyle for Bezier {
    fn path(
        a: RelativeAttachment<Self>,
        a_point: Point,
        b: RelativeAttachment<Self>,
        b_point: Point,
    ) -> Path {
        Path::new(|builder| {
            let halfway_vector = (b_point - a_point) * 0.5;

            let halfway_vector_length =
                (halfway_vector.x.powi(2) + halfway_vector.y.powi(2)).sqrt();

            let halfway_direction = -halfway_vector * (1.0 / halfway_vector_length);

            let a_direction = match a {
                RelativeAttachment::Center => {
                    Vector::new(-halfway_direction.x, halfway_direction.y)
                }
                RelativeAttachment::Edge {
                    edge: Edge::Top, ..
                } => Vector::new(0.0, 1.0),
                RelativeAttachment::Edge {
                    edge: Edge::Bottom, ..
                } => Vector::new(0.0, -1.0),
                RelativeAttachment::Edge {
                    edge: Edge::Left, ..
                } => Vector::new(-1.0, 0.0),
                RelativeAttachment::Edge {
                    edge: Edge::Right, ..
                } => Vector::new(1.0, 0.0),
            };

            let b_direction = match b {
                RelativeAttachment::Center => {
                    Vector::new(halfway_direction.x, -halfway_direction.y)
                }
                RelativeAttachment::Edge {
                    edge: Edge::Top, ..
                } => Vector::new(0.0, 1.0),
                RelativeAttachment::Edge {
                    edge: Edge::Bottom, ..
                } => Vector::new(0.0, -1.0),
                RelativeAttachment::Edge {
                    edge: Edge::Left, ..
                } => Vector::new(-1.0, 0.0),
                RelativeAttachment::Edge {
                    edge: Edge::Right, ..
                } => Vector::new(1.0, 0.0),
            };

            let ctrl_1 = Point::new(
                a_point.x + halfway_vector_length * a_direction.x,
                a_point.y - halfway_vector_length * a_direction.y,
            );

            let ctrl_2 = Point::new(
                b_point.x + halfway_vector_length * b_direction.x,
                b_point.y - halfway_vector_length * b_direction.y,
            );

            builder.move_to(a_point);
            builder.bezier_curve_to(ctrl_1, ctrl_2, b_point);
        })
    }
}
