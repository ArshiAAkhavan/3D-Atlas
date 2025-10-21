use crate::error::{AtlasError, Result};

/// Layer represents a Layer in the Scene Graph.
/// It is a self contained Graph of G(V,E)
#[derive(Debug, Default, Clone)]
pub struct Layer {
    pub nodes: Vec<Node>,
}

impl Layer {
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn del_node(&mut self, id: usize) -> Result<Node> {
        let pos = self
            .nodes
            .iter()
            .position(|n| n.id == id)
            .ok_or(AtlasError::NodeNotFound)?;
        let node = self.nodes.swap_remove(pos);

        // Remove all edges connected to the node
        for node in self.nodes.iter_mut() {
            node.edges.retain(|e| e.dst != id);
        }
        Ok(node)
    }

    /// Add a directed edge between to nodes in the Layer.
    /// If src or dst isn't present in the Layer, it would result in an Error
    pub fn add_edge(&mut self, src: usize, edge: Edge) -> Result<()> {
        let _ = self
            .nodes
            .iter()
            .find(|n| n.id == edge.dst)
            .ok_or(AtlasError::NodeNotFound)?;
        let src_node = self
            .nodes
            .iter_mut()
            .find(|n| n.id == src)
            .ok_or(AtlasError::NodeNotFound)?;
        src_node.edges.push(edge);
        Ok(())
    }

    /// Deletes an edge between `src` and `dst`.
    /// If src doesn't exist or there is no edge between src and dst, it would return an error
    pub fn del_edge(&mut self, src: usize, dst: usize) -> Result<()> {
        let src_node = self
            .nodes
            .iter_mut()
            .find(|n| n.id == src)
            .ok_or(AtlasError::NodeNotFound)?;
        let edge_pos = src_node
            .edges
            .iter()
            .position(|e| e.dst == dst)
            .ok_or(AtlasError::EdgeNotFound)?;

        src_node.edges.swap_remove(edge_pos);
        Ok(())
    }

    pub fn node(&self, id: usize) -> Result<&Node> {
        self.nodes
            .iter()
            .find(|n| n.id == id)
            .ok_or(AtlasError::NodeNotFound)
    }

    pub fn node_mut(&mut self, id: usize) -> Result<&mut Node> {
        self.nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or(AtlasError::NodeNotFound)
    }

    pub fn nodes_with_features(&self, features: &[Feature]) -> Vec<&Node> {
        self.nodes
            .iter()
            .filter(|n| features.iter().all(|f| n.features.contains(f)))
            .collect()
    }

    pub fn edges_matching(&self, desc: &str) -> Vec<EdgeView> {
        self.nodes
            .iter()
            .flat_map(|n| {
                n.edges
                    .iter()
                    .zip(std::iter::repeat(n.id))
                    .filter(|(e, _)| e.meta.desc == desc)
                    .map(|(e, src)| EdgeView { src, edge: e })
            })
            .collect()
    }

    /// Edges going out of a certain node
    pub fn edges_from(&self, src: usize) -> Vec<&Edge> {
        self.nodes
            .iter()
            .filter(|n| n.id == src)
            .flat_map(|n| n.edges.iter())
            .collect()
    }

    /// Edges entering a certain node
    pub fn edges_to(&self, dst: usize) -> Vec<EdgeView> {
        self.nodes
            .iter()
            .flat_map(|n| {
                n.edges
                    .iter()
                    .zip(std::iter::repeat(n.id))
                    .filter(|(e, _)| e.dst == dst)
            })
            .map(|(e, src)| EdgeView { src, edge: e })
            .collect()
    }

    /// Merges an update sub graph into the current Layer.
    /// For each node in the update:
    /// - If the node exists in the current Layer:
    ///   - append new features (avoid duplicates)
    ///   - update existing features if the key matches
    ///   - append new edges (avoid duplicates)
    ///   - update existing edges descriptions if dst matches
    ///   - append new children_ids (avoid duplicates)
    ///   - update parent_id if it is not None
    /// - If the node does not exist, add it directly to the current Layer.
    pub fn merge(&mut self, update: Layer) -> Result<()> {
        let mut new_edges = Vec::new();
        for new_node in update.nodes {
            match self.node_mut(new_node.id) {
                Ok(old_node) => {
                    // Merge features
                    for feature in new_node.features.into_iter() {
                        match old_node.get_feature_mut(feature.key()) {
                            Some(f) => *f = feature,
                            None => old_node.features.push(feature),
                        }
                    }
                    // Merge edges
                    for edge in new_node.edges.into_iter() {
                        match old_node.get_edges_to_mut(edge.dst) {
                            Some(e) => *e = edge,
                            None => new_edges.push((old_node.id, edge)),
                        }
                    }
                }
                Err(_) => {
                    // Node doesn't exist, add it directly
                    self.add_node(new_node);
                }
            }
        }
        // Add new edges after processing all nodes to avoid borrowing issues
        new_edges
            .into_iter()
            .try_for_each(|(src, edge)| self.add_edge(src, edge))
    }
}

#[derive(Debug, Default, Clone)]
pub struct Node {
    /// Unique ID of the node
    pub id: usize,

    /// Unique ID of the parent (if available).
    /// Note that parent does not exist in the Layer
    pub parent_id: Option<usize>,

    pub edges: Vec<Edge>,

    /// List of Node IDs in the bottem layer that are nested under
    /// this node.
    pub children_ids: Vec<usize>,

    pub features: Vec<Feature>,
}

impl Node {
    pub fn has_feature(&self, label: &str) -> bool {
        self.features.iter().map(Feature::key).any(|k| k == label)
    }
    pub fn get_feature(&self, label: &str) -> Option<&Feature> {
        self.features.iter().find(|f| f.key() == label)
    }

    pub fn get_feature_mut(&mut self, label: &str) -> Option<&mut Feature> {
        self.features.iter_mut().find(|f| f.key() == label)
    }

    fn get_edges_to(&self, dst: usize) -> Option<&Edge> {
        self.edges.iter().find(|e| e.dst == dst)
    }

    fn get_edges_to_mut(&mut self, dst: usize) -> Option<&mut Edge> {
        self.edges.iter_mut().find(|e| e.dst == dst)
    }
}

/// Feature represents a single datum in the node.
#[derive(Debug, Clone, PartialEq)]
pub enum Feature {
    /// SemanticLabel represent textual/semantic datum for the node
    SemanticLabel { key: String, value: String },

    /// MetricLabel represent numerical datum for the node
    MetricLabel { key: String, value: f32 },
}

impl Feature {
    pub fn key(&self) -> &str {
        match self {
            Feature::SemanticLabel { key, .. } => key,
            Feature::MetricLabel { key, .. } => key,
        }
    }

    pub fn semantic(key: &str, value: &str) -> Self {
        Feature::SemanticLabel {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
    pub fn metric(key: &str, value: f32) -> Self {
        Feature::MetricLabel {
            key: key.to_string(),
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub dst: usize,
    pub meta: EdgeMeta,
}

pub struct EdgeView<'a> {
    pub src: usize,
    pub edge: &'a Edge,
}

#[derive(Debug, Clone)]
struct EdgeMeta {
    pub desc: String,
}

impl Edge {
    pub fn new(dst: usize, desc: &str) -> Self {
        Edge {
            dst,
            meta: EdgeMeta {
                desc: desc.to_string(),
            },
        }
    }
}
