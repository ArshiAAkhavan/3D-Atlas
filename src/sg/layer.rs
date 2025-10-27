use super::{Edge, Node};
use crate::error::{AtlasError, Result};


/// A Layer in the Scene Graph containing multiple Nodes and their Edges.
/// Each Layer is a well-defined Graph structure representing a specific aspect of the scene,
/// such as semantic relationships or physical connections between objects.
#[derive(Debug, Clone)]
pub struct Layer {
    /// List of nodes in this layer.
    pub nodes: Vec<Node>,
}

impl Layer {
    pub(super) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Get a reference to a node by its ID.
    pub fn node(&self, id: usize) -> Result<&Node> {
        self.nodes
            .iter()
            .find(|node| node.id == id)
            .ok_or(AtlasError::NodeNotFound)
    }

    /// Get a mutable reference to a node by its ID.
    pub fn node_mut(&mut self, id: usize) -> Result<&mut Node> {
        self.nodes
            .iter_mut()
            .find(|node| node.id == id)
            .ok_or(AtlasError::NodeNotFound)
    }

    /// Delete a node by its ID, removing all associated edges in the layer.
    pub(super) fn del_node(&mut self, id: usize) -> Result<Node> {
        let index = self
            .nodes
            .iter()
            .position(|node| node.id == id)
            .ok_or(AtlasError::NodeNotFound)?;
        let node = self.nodes.remove(index);
        self.nodes
            .iter_mut()
            .for_each(|node| node.edges.retain(|edge| edge.dst != id));
        Ok(node)
    }

    /// Add a new node to the layer.
    pub fn push_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Add an edge from source node to destination node with a description.
    /// Ensures both source and destination nodes exist in the layer.
    pub fn add_edge(&mut self, src: usize, dst: usize, desc: &str) -> Result<()> {
        // Ensure destination node exists
        let _ = self.node(dst)?;
        let src_node = self.node_mut(src)?;
        src_node.edges.push(Edge::new(src, dst, desc));
        Ok(())
    }

    /// Delete an edge from source node to destination node.
    /// Returns an error if the edge does not exist.
    pub fn del_edge(&mut self, src: usize, dst: usize) -> Result<()> {
        let src_node = self.node_mut(src)?;
        let index = src_node
            .edges
            .iter()
            .position(|edge| edge.dst == dst)
            .ok_or(AtlasError::EdgeNotFound)?;
        src_node.edges.swap_remove(index);
        Ok(())
    }
}

/// Query
impl Layer {
    /// Get List of all nodes matching a specific node features.
    pub fn nodes_having(&self, keys: &[&str]) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter(|node| keys.iter().all(|key| node.has_feature(key)))
            .collect()
    }

    /// Get List of all nodes matching a specific node features.
    pub fn nodes_matching(&self, features: &[&super::node::Feature]) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter(|node| features.iter().all(|f| node.match_feature(f)))
            .collect()
    }

    /// Get List of all edges matching a specific description.
    pub fn edges_matching(&self, desc: &str) -> Vec<&Edge> {
        self.nodes
            .iter()
            .flat_map(|n| n.edges.iter().filter(|e| e.desc == desc))
            .collect()
    }

    /// Get List of all edges from a specific source node.
    pub fn edges_from(&self, src: usize) -> Vec<&Edge> {
        match self.node(src) {
            Ok(n) => n.edges.iter().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Get List of all edges to a specific destination node.
    pub fn edges_to(&self, dst: usize) -> Vec<&Edge> {
        self.nodes
            .iter()
            .flat_map(|n| n.edges.iter().filter(|e| e.dst == dst))
            .collect()
    }
}
impl Layer {

    /// Merge another layer into this one.
    /// Nodes with the same ID will be merged, while new nodes will be added.
    /// Deleting Nodes and edges is not supported in this operation.
    pub fn merge(&mut self, l2: Layer) -> std::result::Result<(), AtlasError> {
        for node in l2.nodes {
            match self.node_mut(node.id) {
                Ok(existing_node) => {
                    existing_node.merge(node)?;
                }
                Err(AtlasError::NodeNotFound) => {
                    self.push_node(node.clone());
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
