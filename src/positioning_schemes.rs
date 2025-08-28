use crate::graph::connections::Edge;
use crate::graph::line_styles::AxisAligned;
use crate::graph::{GraphNode, RelativeAttachment, line_styles};
use crate::{Node, widgets::*};

use iced::{Rectangle, Size, Vector};

use crate::graph::GraphData;

#[allow(clippy::type_complexity)]
pub fn family_tree(
    prev: Option<(
        usize,
        &GraphNode<Node>,
        &RelativeAttachment<AxisAligned>,
        &RelativeAttachment<AxisAligned>,
        Size<f32>,
    )>,
    id: usize,
    node: &GraphNode<Node>,
    size: iced::Size,
    data: &GraphData<Node, RelativeAttachment<line_styles::AxisAligned>>,
    layout: &iced::advanced::Layout<'_>,
    visited: Vec<usize>,
) -> Vector {
    let Some((prev_id, prev, attachment, prev_attachment, prev_size)) = prev else {
        let total_covered_space = visited
            .iter()
            .filter_map(|other_id| {
                (other_id != &id).then_some(()).and_then(|_| {
                    layout
                        .children()
                        .nth(*other_id)
                        .map(|layout| layout.bounds())
                })
            })
            .fold(
                Rectangle::new(layout.position(), Size::ZERO),
                |acc, bounds| acc.union(&bounds),
            );

        return Vector::new(total_covered_space.width + 75.0, 0.0);
    };

    match node.data() {
        Node::Character(_) => match prev.data() {
            Node::Character(_) => unreachable!(),
            Node::Family => match prev_attachment {
                RelativeAttachment::Edge {
                    edge: Edge::Bottom, ..
                } => {
                    let mut children: Vec<_> = data
                        .get_connections(prev_id)
                        .filter_map(|conn| {
                            matches!(
                                conn.0,
                                RelativeAttachment::Edge {
                                    edge: Edge::Bottom,
                                    ..
                                }
                            )
                            .then_some((
                                conn.1,
                                layout.children().nth(conn.1).unwrap().bounds().size(),
                                None,
                            ))
                        })
                        .collect();

                    let num_visited_partners =
                        add_partners_to_children(&mut children, data, layout, &visited) as f32;

                    let padding = 25.0;

                    let leftmost = -(children
                        .iter()
                        .fold(0.0, |acc, (_, size, _)| acc + size.width)
                        + (children.len() as f32 - 1.0) * padding)
                        / 2.0;

                    let visited_children: Vec<_> = children
                        .iter()
                        .filter(|(child_id, _, partner_edge)| {
                            visited.contains(child_id)
                                || partner_edge.as_ref().is_some_and(|partners_partner| {
                                    visited.contains(partners_partner)
                                })
                        })
                        .collect();

                    let x = leftmost
                        + visited_children
                            .iter()
                            .fold(0.0, |acc, (_, size, _)| acc + size.width + padding)
                        + padding * num_visited_partners;

                    Vector::new(x, 75.0)
                }
                RelativeAttachment::Edge {
                    edge: Edge::Top, ..
                } => {
                    let x = match attachment {
                        RelativeAttachment::Edge {
                            edge: Edge::Left, ..
                        } => 30.0,
                        RelativeAttachment::Edge {
                            edge: Edge::Right, ..
                        } => -20.0 - size.width,
                        _ => unreachable!(),
                    };

                    Vector::new(x, -25.0 - size.height)
                }
                _ => unreachable!(),
            },
        },
        Node::Family => {
            if let Node::Character(_) = prev.data() {
                match prev_attachment {
                    RelativeAttachment::Edge {
                        edge: Edge::Left, ..
                    } => Vector::new(-30.0, prev_size.height + 25.0),
                    RelativeAttachment::Edge {
                        edge: Edge::Right, ..
                    } => Vector::new(20.0 + prev_size.width, prev_size.height + 25.0),
                    RelativeAttachment::Edge {
                        edge: Edge::Top, ..
                    } => {
                        let mut children: Vec<_> = data
                            .get_connections(id)
                            .filter_map(|conn| {
                                matches!(
                                    conn.0,
                                    RelativeAttachment::Edge {
                                        edge: Edge::Bottom,
                                        ..
                                    }
                                )
                                .then_some((
                                    conn.1,
                                    layout.children().nth(conn.1).unwrap().bounds().size(),
                                    false,
                                ))
                            })
                            .collect();

                        let mut num_partners = 0.0;

                        for (i, (child_id, _, _)) in children.clone().iter().enumerate() {
                            for (att, other_id, _) in data.get_connections(*child_id) {
                                if let Node::Family = data.get(other_id).unwrap().data()
                                    && att.is_horizontal()
                                {
                                    let Some(partner_id) = data.get_connections(other_id).find_map(
                                        |(family_att, partner_id, _)| {
                                            (family_att.is_top() && other_id != *child_id)
                                                .then_some(partner_id)
                                        },
                                    ) else {
                                        continue;
                                    };

                                    let partner_size =
                                        layout.children().nth(partner_id).unwrap().bounds().size();

                                    let elem = (partner_id, partner_size, true);

                                    if !children.contains(&elem) {
                                        if att.is_right() && i < children.len() {
                                            children.insert(i, elem);
                                            num_partners += 1.0;
                                        } else if att.is_left() {
                                            children.insert(i + 1, elem);
                                            num_partners += 1.0;
                                        }
                                    }
                                }
                            }
                        }

                        let padding = 25.0;

                        let children_width = children
                            .iter()
                            .fold(0.0, |acc, (_, size, _)| acc + size.width)
                            + (children.len() as f32 + num_partners) * padding;

                        Vector::new(children_width / 2.0, -75.0)
                    }
                    _ => unreachable!(),
                }
            } else {
                unreachable!()
            }
        }
    }
}

fn add_partners_to_children<S: graph::line_styles::LineStyle + PartialEq + Send>(
    children: &mut Vec<(usize, Size, Option<usize>)>,
    data: &GraphData<Node, RelativeAttachment<S>>,
    layout: &iced::advanced::Layout<'_>,
    visited: &[usize],
) -> usize {
    let mut num_visited_partners = 0;

    for (i, (child_id, _, _)) in children.clone().iter().enumerate() {
        for (att, other_id, _) in data.get_connections(*child_id) {
            if let Node::Family = data.get(other_id).unwrap().data()
                && att.is_horizontal()
            {
                let Some(partner_id) =
                    data.get_connections(other_id)
                        .find_map(|(family_att, partner_id, _)| {
                            (family_att.is_top() && other_id != *child_id).then_some(partner_id)
                        })
                else {
                    continue;
                };

                let partner_size = layout.children().nth(partner_id).unwrap().bounds().size();

                if !children.iter().any(|(id, _, _)| *id == partner_id) {
                    if att.is_right() && i < children.len() {
                        children.insert(i + 1, (partner_id, partner_size, Some(*child_id)));
                        if visited.contains(child_id) {
                            num_visited_partners += 1;
                        }
                    } else if att.is_left() {
                        children.insert(i, (partner_id, partner_size, Some(*child_id)));
                        if visited.contains(child_id) {
                            num_visited_partners += 1;
                        }
                    }
                }
            }
        }
    }

    num_visited_partners
}
