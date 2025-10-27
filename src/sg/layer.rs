use super::{Edge, Node};
use crate::error::{AtlasError, Result};

#[derive(Debug, Clone)]
pub struct Layer {
    pub nodes: Vec<Node>,
}

impl Layer {
    pub(super) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn node(&self, id: usize) -> Result<&Node> {
        self.nodes
            .iter()
            .find(|node| node.id == id)
            .ok_or(AtlasError::NodeNotFound)
    }

    pub fn node_mut(&mut self, id: usize) -> Result<&mut Node> {
        self.nodes
            .iter_mut()
            .find(|node| node.id == id)
            .ok_or(AtlasError::NodeNotFound)
    }

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

    pub fn push_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, src: usize, dst: usize, desc: &str) -> Result<()> {
        // Ensure destination node exists
        let _ = self.node(dst)?;
        let src_node = self.node_mut(src)?;
        src_node.edges.push(Edge::new(src, dst, desc));
        Ok(())
    }

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

    /// Get List of all nodes matching a specific node features.
    pub fn nodes_having_features(&self, keys: &[&str]) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter(|node| keys.iter().all(|key| node.has_feature(key)))
            .collect()
    }

    /// Get List of all nodes matching a specific node features.
    pub fn nodes_matching_features(&self, features: &[&super::node::Feature]) -> Vec<&Node> {
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
