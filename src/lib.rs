use serde::{Deserialize, Serialize};

mod error;
mod node;

use crate::error::{AtlasError, Result};
use node::Node;

/// A live scene graph containing multiple snapshots of the environment.
/// Each snapshot represents the state of the scene graph at a specific point in time.
pub struct LiveSceneGraph {
    /// A list of snapshots in the live scene graph.
    pub snapshots: Vec<Snapshot>,
}

impl LiveSceneGraph {
    /// Create a new empty LiveSceneGraph.
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    /// Check if a node with the given index exists in the latest the SceneGraph
    pub fn has_node(&self, index: usize) -> bool {
        self.snapshots
            .iter()
            .any(|s| s.nodes.iter().any(|n| n.id == index))
    }

    /// Add a snapshot to the live scene graph.
    pub fn add_snapshot(&mut self, snapshot: Snapshot) {
        self.snapshots.push(snapshot);
    }

    /// Add a new edge to the latest snapshot in the live scene graph and create a new snapshot.
    pub fn add_edge(&mut self, src: usize, dst: usize, meta: EdgeMeta) -> Result<()> {
        if !self.has_node(src) || !self.has_node(dst) {
            return Err(AtlasError::NodeNotFound);
        }
        let mut current_snapshot = self.now().clone();
        current_snapshot.add_edge(src, dst, meta);
        self.snapshots.push(current_snapshot);
        Ok(())
    }

    /// Delete an edge from the latest snapshot in the live scene graph.
    /// If the edge does not exist, do nothing.
    pub fn del_edge(&mut self, src: usize, dst: usize, meta: EdgeMeta) -> Result<()> {
        let mut current_snapshot = self.now().clone();
        match current_snapshot.del_edge(src, dst, meta) {
            Err(AtlasError::EdgeNotFound) => Ok(()),
            Ok(_) => {
                self.snapshots.push(current_snapshot);
                Ok(())
            }
            Err(e) => Err(e),
        }
        //TODO: (ArshiA) Reflect "do nothing" at type-system level
    }

    /// Update an edge in the latest snapshot in the live scene graph.
    /// If the edge does not exist, acts like [`add_edge`].
    ///
    /// [`add_edge`]: LiveSceneGraph::add_edge
    pub fn update_edge(
        &mut self,
        src: usize,
        dst: usize,
        old_meta: EdgeMeta,
        new_meta: EdgeMeta,
    ) -> Result<()> {
        let mut current_snapshot = self.now().clone();
        match current_snapshot.del_edge(src, dst, old_meta) {
            Err(AtlasError::EdgeNotFound) | Ok(_) => {
                current_snapshot.add_edge(src, dst, new_meta);
                self.snapshots.push(current_snapshot);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Get the latest snapshot in the live scene graph.
    pub fn now(&mut self) -> &Snapshot {
        if self.snapshots.is_empty() {
            self.snapshots.push(Default::default());
        }
        // SAFETY: We just ensured that snapshots is not empty.
        self.snapshots.last().unwrap()
    }
}

impl Default for LiveSceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// A snapshot of the scene graph at a specific point in time.
/// It contains nodes and edges representing objects and their relationships.
/// It also includes metadata such as the number of objects and small objects.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Snapshot {
    /// Indices of small objects in the nodes list.
    pub small_objects: Vec<usize>,

    /// Total number of objects in the snapshot.
    pub num_objects: usize,

    /// List of nodes in the snapshot.
    pub nodes: Vec<Node>,

    /// List of edges in the snapshot.
    pub edges: Vec<Edge>,
}

impl Snapshot {
    /// Delete an edge from the snapshot.
    /// If the edge does not exist, do nothing.
    pub fn del_edge(&mut self, src: usize, dst: usize, meta: EdgeMeta) -> Result<()> {
        let len = self.edges.len();
        self.edges
            .retain(|e| !(e.src == src && e.dst == dst && e.meta == meta));

        if len == self.edges.len() {
            Err(AtlasError::EdgeNotFound)
        } else {
            Ok(())
        }
    }

    /// Add an edge to the snapshot.
    /// src and dst may not appear in the current snapshot's nodes list.
    pub fn add_edge(&mut self, src: usize, dst: usize, meta: EdgeMeta) {
        self.edges.push(Edge { src, dst, meta })
    }
}

/// An edge in the scene graph representing a relationship between two nodes.
/// The edge can represent both hierarchical and relational relation between two nodes .
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Edge {
    /// Source node index.
    pub src: usize,

    /// Destination node index.
    pub dst: usize,

    /// Metadata associated with the edge.
    pub meta: EdgeMeta,
}

/// Metadata for an edge in the scene graph.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct EdgeMeta {
    /// Type of the edge (e.g., "parent", "on", "next_to").
    pub desc: String,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn snapshot_edge_crud() {
        let mut s = Snapshot::default();
        let meta1 = EdgeMeta {
            desc: "parent".to_string(),
        };
        let meta2 = EdgeMeta {
            desc: "on".to_string(),
        };

        // add edges
        s.add_edge(0, 1, meta1.clone());
        s.add_edge(1, 2, meta2.clone());
        assert_eq!(s.edges.len(), 2);

        let e1 = Edge {
            src: 0,
            dst: 1,
            meta: meta1.clone(),
        };
        let e2 = Edge {
            src: 1,
            dst: 2,
            meta: meta2.clone(),
        };
        assert!(s.edges.contains(&e1));
        assert!(s.edges.contains(&e2));

        // delete edges
        assert!(s.del_edge(0, 1, meta1.clone()).is_ok());
        assert_eq!(s.edges.len(), 1);
        assert!(!s.edges.contains(&e1));
        assert!(s.edges.contains(&e2));

        // delete non-existing edges
        assert!(s.del_edge(0, 1, meta1.clone()).is_err());
        assert_eq!(s.edges.len(), 1);
        assert!(!s.edges.contains(&e1));
        assert!(s.edges.contains(&e2));
    }

    #[test]
    fn scene_graph_edge_crud() {
        let mut g = LiveSceneGraph::new();
        let em1 = EdgeMeta {
            desc: "parent".to_string(),
        };
        let em2 = EdgeMeta {
            desc: "on".to_string(),
        };
        let em3 = EdgeMeta {
            desc: "next_to".to_string(),
        };

        let e1 = Edge {
            src: 0,
            dst: 1,
            meta: em1.clone(),
        };
        let e2 = Edge {
            src: 1,
            dst: 2,
            meta: em2.clone(),
        };
        let e3 = Edge {
            src: 1,
            dst: 2,
            meta: em3.clone(),
        };
        let e4 = Edge {
            src: 0,
            dst: 2,
            meta: em2.clone(),
        };

        let node1 = Node::new(0);
        let node2 = Node::new(1);
        let node3 = Node::new(2);
        let snapshot = Snapshot {
            small_objects: vec![],
            num_objects: 3,
            nodes: vec![node1, node2, node3],
            edges: vec![],
        };

        g.add_snapshot(snapshot);
        assert_eq!(g.snapshots.len(), 1);
        assert!(g.has_node(0));
        assert!(g.has_node(1));
        assert!(g.has_node(2));
        assert!(!g.has_node(3));

        // add edges
        assert!(g.add_edge(e1.src, e1.dst, em1.clone()).is_ok());
        assert_eq!(g.snapshots.len(), 2);
        assert!(g.now().edges.contains(&e1));
        assert!(g.add_edge(e2.src, e2.dst, em2.clone()).is_ok());
        assert_eq!(g.snapshots.len(), 3);
        assert!(g.now().edges.contains(&e2));

        // add edge with non-existing node
        assert!(g.add_edge(2, 3, em1.clone()).is_err());
        // no new snapshot should be created
        assert_eq!(g.snapshots.len(), 3);

        // delete edges
        assert!(g.del_edge(e1.src, e1.dst, em1.clone()).is_ok());
        assert_eq!(g.snapshots.len(), 4);
        assert!(!g.now().edges.contains(&e1));
        assert!(g.now().edges.contains(&e2));

        // delete non-existing edges
        assert!(g.del_edge(0, 1, em1.clone()).is_ok());
        // no new snapshot should be created
        assert_eq!(g.snapshots.len(), 4);
        assert!(!g.now().edges.contains(&e1));
        assert!(g.now().edges.contains(&e2));

        // update edges
        assert!(
            g.update_edge(e2.src, e2.dst, em2.clone(), em3.clone())
                .is_ok()
        );
        assert_eq!(g.snapshots.len(), 5);
        assert!(!g.now().edges.contains(&e2));
        assert!(g.now().edges.contains(&e3));

        // update non-existing edges (acts like add_edge)
        assert!(g.update_edge(0, 2, em1.clone(), em2.clone()).is_ok());
        assert_eq!(g.snapshots.len(), 6);
        assert!(g.now().edges.contains(&e4));
    }
}
