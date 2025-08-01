use std::collections::VecDeque;

use iced::Point;

use crate::graph::connections::{self, Attachment, Connection, RelativeAttachment};

#[derive(Debug, Clone)]
pub struct GraphNode<D: std::fmt::Debug> {
    pub(super) position: Point,
    pub(super) data: D,
}

impl<Data: std::fmt::Debug> GraphNode<Data> {
    pub fn new(position: Point, data: Data) -> Self {
        Self { position, data }
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn move_to(&mut self, position: Point) {
        self.position = position;
    }
}

pub struct GraphData<
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + std::cmp::PartialEq = RelativeAttachment,
> {
    pub(super) nodes: Vec<GraphNode<Data>>,
    pub(super) connections: Vec<Connection<Attachment>>,
}

impl<Data: std::fmt::Debug, Attachment: connections::Attachment + std::cmp::PartialEq> Default
    for GraphData<Data, Attachment>
{
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }
}

impl<Data, Attachment> GraphData<Data, Attachment>
where
    Data: std::fmt::Debug,
    Attachment: self::Attachment + std::cmp::PartialEq,
{
    pub fn get(&self, id: usize) -> Result<&GraphNode<Data>, GraphError> {
        self.nodes.get(id).ok_or(GraphError::NodeNotFound(id))
    }

    pub fn get_mut(&mut self, id: usize) -> Result<&mut GraphNode<Data>, GraphError> {
        self.nodes.get_mut(id).ok_or(GraphError::NodeNotFound(id))
    }

    pub fn add(&mut self, node: Data, position: Point) {
        self.nodes.push(GraphNode::new(position, node));
    }

    pub fn add_to(
        &mut self,
        node: Data,
        position: Point,
        attachment: Attachment,
        connection: usize,
        other_attachment: Attachment,
    ) -> Result<(), GraphError> {
        if self.nodes.len() < connection {
            return Result::Err(GraphError::NodeNotFound(connection));
        }

        self.add(node, position);

        self.connect(
            self.nodes.len() - 1,
            attachment,
            connection,
            other_attachment,
        )
    }

    pub fn get_connections(&self, id: usize) -> Vec<&GraphNode<Data>> {
        self.connections
            .iter()
            .filter_map(|Connection { a, b, .. }| {
                if a.0 == id {
                    Some(self.get(b.0).unwrap())
                } else if b.0 == id {
                    Some(self.get(a.0).unwrap())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn connect(
        &mut self,
        a: usize,
        a_attachment: Attachment,
        b: usize,
        b_attachment: Attachment,
    ) -> Result<(), GraphError> {
        if self.nodes.len() < a {
            return Result::Err(GraphError::NodeNotFound(a));
        }

        if self.nodes.len() < b {
            return Result::Err(GraphError::NodeNotFound(b));
        }

        if a == b && a_attachment == b_attachment {
            return Ok(());
        }

        self.connections
            .push(Connection::new(a, a_attachment, b, b_attachment));

        Ok(())
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn disconnect(&mut self, i: usize) {
        self.connections.remove(i);
    }

    pub fn remove(&mut self, i: usize) {
        self.nodes.remove(i);
        self.connections
            .retain(|conn| conn.a.0 != i && conn.b.0 != i);

        self.connections.iter_mut().for_each(|conn| {
            if conn.a.0 > i {
                conn.a.0 -= 1;
            }
            if conn.b.0 > i {
                conn.b.0 -= 1;
            }
        });
    }

    pub fn traverse<F>(&self, starting_node: Option<usize>, f: F)
    where
        F: Fn(usize, &GraphNode<Data>),
    {
        let mut visited = Vec::new();

        let mut stack = VecDeque::with_capacity(self.nodes.len());

        stack.push_back(starting_node.unwrap_or(0));

        if *stack.iter().next().unwrap() >= self.nodes.len() {
            return;
        }

        while !stack.is_empty() {
            if visited.len() == self.nodes.len() {
                break;
            }

            let current_node = *stack.iter().last().unwrap();

            if !visited.contains(&current_node) {
                f(current_node, &self.nodes[current_node]);
                visited.push(current_node);
            }

            let mut unvisited_connections = self.connections.iter().filter_map(|conn| {
                (conn.a.0 == current_node)
                    .then_some(conn.b.0)
                    .or_else(|| (conn.b.0 == current_node).then_some(conn.a.0))
                    .filter(|conn| !visited.contains(conn))
            });

            if let Some(next_node) = unvisited_connections.next() {
                stack.push_back(next_node);
            } else {
                stack.pop_back();
            }
        }
    }

    pub fn traverse_breadth_first<F>(&self, starting_node: Option<usize>, f: F)
    where
        F: Fn(usize, &GraphNode<Data>),
    {
        let mut visited = Vec::new();

        let mut queue = VecDeque::with_capacity(self.nodes.len());

        let starting_node = starting_node.unwrap_or(0);

        if starting_node >= self.nodes.len() {
            return;
        }

        queue.push_back(starting_node);
        visited.push(starting_node);
        f(starting_node, &self.nodes[starting_node]);

        while !queue.is_empty() {
            if visited.len() == self.nodes.len() {
                break;
            }

            let current_node = *queue.front().unwrap();

            let unvisited_connections: Vec<_> = self
                .connections
                .iter()
                .filter_map(|conn| {
                    (conn.a.0 == current_node)
                        .then_some(conn.b.0)
                        .or_else(|| (conn.b.0 == current_node).then_some(conn.a.0))
                        .filter(|conn| !visited.contains(conn))
                })
                .collect();

            queue.extend(unvisited_connections.iter());
            visited.extend(unvisited_connections.iter());

            for node in &unvisited_connections {
                f(*node, &self.nodes[*node]);
            }

            queue.pop_front();
        }
    }
}

#[derive(Debug)]
pub enum GraphError {
    NodeNotFound(usize),
}
