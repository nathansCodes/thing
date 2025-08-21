use std::collections::VecDeque;

use iced::Point;
use serde::{Deserialize, Serialize};

use crate::graph::{
    connections::{self, Attachment, Connection, RelativeAttachment},
    iter::{BreadthFirstIterator, DepthFirstIterator},
};

#[derive(Serialize, Deserialize)]
#[serde(remote = "Point")]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode<D: std::fmt::Debug> {
    #[serde(with = "Position")]
    pub(super) position: Point,
    pub(super) data: D,
    #[serde(skip)]
    pub(super) selected: bool,
}

impl<Data: std::fmt::Debug> GraphNode<Data> {
    pub fn new(position: Point, data: Data) -> Self {
        Self {
            position,
            data,
            selected: false,
        }
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Data {
        &mut self.data
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub fn move_to(&mut self, position: Point) {
        self.position = position;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphData<Data, Attachment = RelativeAttachment>
where
    Data: std::fmt::Debug,
    Attachment: connections::Attachment + std::cmp::PartialEq,
{
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
    pub fn get(&self, id: usize) -> Option<&GraphNode<Data>> {
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut GraphNode<Data>> {
        self.nodes.get_mut(id)
    }

    pub fn add(&mut self, node: Data, position: Point) {
        self.nodes.push(GraphNode::new(position, node));
    }

    pub fn attach_new(
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

    pub fn get_connected_nodes(&self, id: usize) -> Vec<&GraphNode<Data>> {
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

    pub fn get_connections(
        &self,
        id: usize,
    ) -> impl Iterator<Item = (&Attachment, usize, &Attachment)> {
        self.connections.iter().filter_map(move |conn| {
            (conn.a.0 == id)
                .then_some((&conn.a.1, conn.b.0, &conn.b.1))
                .or_else(|| (conn.b.0 == id).then_some((&conn.b.1, conn.a.0, &conn.a.1)))
        })
    }

    pub fn get_connections_indexed(
        &self,
        id: usize,
    ) -> impl Iterator<Item = (usize, &Attachment, usize, &Attachment)> {
        self.connections
            .iter()
            .enumerate()
            .filter_map(move |(i, conn)| {
                (conn.a.0 == id)
                    .then_some((i, &conn.a.1, conn.b.0, &conn.b.1))
                    .or_else(|| (conn.b.0 == id).then_some((i, &conn.b.1, conn.a.0, &conn.a.1)))
            })
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

    pub fn disconnect_all(&mut self, a: usize, b: usize) {
        self.connections
            .retain(|conn| !((conn.a.0 == a && conn.b.0 == b) || (conn.b.0 == a && conn.a.0 == b)));
    }

    pub fn remove_connection(&mut self, connection_id: usize) {
        self.connections.remove(connection_id);
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

    pub fn is_selected(&self, i: usize) -> Result<bool, GraphError> {
        self.nodes
            .get(i)
            .map(GraphNode::selected)
            .ok_or(GraphError::NodeNotFound(i))
    }

    pub fn select(&mut self, i: usize) {
        if let Some(node) = self.nodes.get_mut(i) {
            node.selected = true;
        }
    }

    pub fn deselect(&mut self, i: usize) {
        if let Some(node) = self.nodes.get_mut(i) {
            node.selected = false;
        }
    }

    pub fn select_all(&mut self) {
        for node in self.nodes.iter_mut() {
            node.selected = true;
        }
    }

    pub fn clear_selection(&mut self) {
        for node in self.nodes.iter_mut() {
            node.selected = false;
        }
    }

    pub fn selection(&self) -> impl Iterator<Item = usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| node.selected.then_some(i))
    }

    pub fn traverse_iter(
        &self,
        starting_node: Option<usize>,
    ) -> impl Iterator<Item = (usize, &GraphNode<Data>)> {
        let mut visited = Vec::new();

        let mut stack = VecDeque::with_capacity(self.nodes.len());

        stack.push_back(starting_node.unwrap_or(0));

        if *stack.iter().next().unwrap() >= self.nodes.len() {
            return Vec::<(usize, &GraphNode<Data>)>::new().into_iter();
        }

        while !stack.is_empty() {
            if visited.len() == self.nodes.len() {
                break;
            }

            let current_node = *stack.iter().last().unwrap();

            if !visited.contains(&current_node) {
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

        visited
            .iter()
            .map(|id| (*id, &self.nodes[*id]))
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn iter(&self) -> impl Iterator<Item = &GraphNode<Data>> {
        self.nodes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut GraphNode<Data>> {
        self.nodes.iter_mut()
    }

    pub fn iter_dfs<'graph: 'iter, 'iter>(
        &'graph self,
        starting_node: usize,
    ) -> DepthFirstIterator<'iter, Data, Attachment> {
        DepthFirstIterator::new(self, starting_node)
    }

    pub fn iter_bfs<'graph: 'iter, 'iter>(
        &'graph self,
        starting_node: usize,
    ) -> BreadthFirstIterator<'iter, Data, Attachment> {
        BreadthFirstIterator::new(self, starting_node)
    }
}

#[derive(Debug)]
pub enum GraphError {
    NodeNotFound(usize),
}
