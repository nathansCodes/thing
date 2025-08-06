use std::collections::VecDeque;

use crate::graph::{GraphData, connections, data::GraphNode};

pub struct DepthFirstIterator<'graph, Data, Attachment>
where
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + std::cmp::PartialEq,
{
    graph_data: &'graph GraphData<Data, Attachment>,
    visited: Vec<usize>,
    stack: VecDeque<usize>,
}

impl<'graph, Data, Attachment> DepthFirstIterator<'graph, Data, Attachment>
where
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + std::cmp::PartialEq,
{
    pub(super) fn new(data: &'graph GraphData<Data, Attachment>, starting_node: usize) -> Self {
        Self {
            graph_data: data,
            visited: Vec::new(),
            stack: [starting_node].into_iter().collect(),
        }
    }
}

impl<'graph, Data, Attachment> Iterator for DepthFirstIterator<'graph, Data, Attachment>
where
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + std::cmp::PartialEq,
{
    type Item = (usize, &'graph GraphNode<Data>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.visited.is_empty() && self.stack.len() == 1 {
            let next = *self.stack.front().unwrap();
            self.visited.push(next);
            return Some((next, &self.graph_data.nodes[next]));
        }

        if self.visited.len() == self.graph_data.nodes.len() {
            return None;
        }

        if self.stack.is_empty() {
            return None;
        }

        let current_node = *self.stack.iter().last().unwrap();

        if !self.visited.contains(&current_node) {
            self.visited.push(current_node);
        }

        let mut unvisited_connections = self.graph_data.connections.iter().filter_map(|conn| {
            (conn.a.0 == current_node)
                .then_some(conn.b.0)
                .or_else(|| (conn.b.0 == current_node).then_some(conn.a.0))
                .filter(|conn| !self.visited.contains(conn))
        });

        if let Some(next_node) = unvisited_connections.next() {
            self.stack.push_back(next_node);
        } else {
            self.stack.pop_back();
        }

        Some((current_node, &self.graph_data.nodes[current_node]))
    }
}

pub struct BreadthFirstIterator<'a, Data, Attachment>
where
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + std::cmp::PartialEq,
{
    graph_data: &'a GraphData<Data, Attachment>,
    visited: Vec<usize>,
    queue: VecDeque<usize>,
}

impl<'graph, Data, Attachment> BreadthFirstIterator<'graph, Data, Attachment>
where
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + std::cmp::PartialEq,
{
    pub(super) fn new(data: &'graph GraphData<Data, Attachment>, starting_node: usize) -> Self {
        Self {
            graph_data: data,
            visited: Vec::new(),
            queue: [starting_node].into_iter().collect(),
        }
    }
}

impl<'a, Data, Attachment> Iterator for BreadthFirstIterator<'a, Data, Attachment>
where
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + std::cmp::PartialEq,
{
    type Item = (usize, &'a GraphNode<Data>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.visited.is_empty() && self.queue.len() == 1 {
            let starting_node = *self.queue.front().unwrap();
            self.visited.push(starting_node);

            return Some((starting_node, &self.graph_data.nodes[starting_node]));
        }

        if self.visited.len() == self.graph_data.nodes.len() {
            return None;
        }

        if self.queue.is_empty() {
            return None;
        }

        let current_node = *self.queue.front().unwrap();

        let unvisited_connections: Vec<_> = self
            .graph_data
            .connections
            .iter()
            .filter_map(|conn| {
                (conn.a.0 == current_node)
                    .then_some(conn.b.0)
                    .or_else(|| (conn.b.0 == current_node).then_some(conn.a.0))
                    .filter(|conn| !self.visited.contains(conn))
            })
            .collect();

        if unvisited_connections.is_empty() {
            self.queue.pop_front();

            return self.next();
        }

        let next = *unvisited_connections.first().unwrap();

        self.queue.push_back(next);
        self.visited.push(next);

        Some((next, &self.graph_data.nodes[next]))
    }
}
