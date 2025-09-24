use serde::{Deserialize, Serialize};

mod error;
mod node;

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

    /// Add a snapshot to the live scene graph.
    pub fn add_snapshot(&mut self, snapshot: Snapshot) {
        self.snapshots.push(snapshot);
    }

    /// Add a new edge to the latest snapshot in the live scene graph and create a new snapshot.
    pub fn add_edge(&mut self, src: usize, dst: usize, meta: EdgeMeta) {
        let mut current_snapshot = match self.snapshots.last() {
            Some(s) => s.clone(),
            None => Default::default(),
        };
        current_snapshot.add_edge(src, dst, meta);
        self.snapshots.push(current_snapshot);
    }

    /// Delete an edge from the latest snapshot in the live scene graph.
    /// If the edge does not exist, do nothing.
    pub fn del_edge(&mut self, src: usize, dst: usize, meta: EdgeMeta) {
        if let Some(current_snapshot) = self.snapshots.last() {
            let mut current_snapshot = current_snapshot.clone();
            current_snapshot.del_edge(src, dst, meta);
            self.snapshots.push(current_snapshot);
        }
        //TODO: (ArshiA) Reflect "do nothing" at type-system level
    }

    /// Update an edge in the latest snapshot in the live scene graph.
    /// If the edge does not exist, acts like [`add_edge`].
    ///
    /// [`add_edge`]: LiveSceneGraph::add_edge
    pub fn update_edge(&mut self, src: usize, dst: usize, old_meta: EdgeMeta, new_meta: EdgeMeta) {
        let mut current_snapshot = match self.snapshots.last() {
            Some(s) => s.clone(),
            None => Default::default(),
        };
        current_snapshot.del_edge(src, dst, old_meta);
        current_snapshot.add_edge(src, dst, new_meta);
        self.snapshots.push(current_snapshot);
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
    pub fn del_edge(&mut self, src: usize, dst: usize, meta: EdgeMeta) {
        self.edges
            .retain(|e| !(e.src == src && e.dst == dst && e.meta == meta))
    }

    /// Add an edge to the snapshot.
    pub fn add_edge(&mut self, src: usize, dst: usize, meta: EdgeMeta) {
        self.edges.push(Edge { src, dst, meta })
    }
}


/// An edge in the scene graph representing a relationship between two nodes.
/// The edge can represent both hierarchical and relational relation between two nodes .
#[derive(Serialize, Deserialize, Debug, Clone)]
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
