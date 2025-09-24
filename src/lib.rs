use serde::{Deserialize, Serialize};

/// A live scene graph containing multiple snapshots of the environment.
/// Each snapshot represents the state of the scene graph at a specific point in time.
pub struct LiveSceneGraph {
    /// A list of snapshots in the live scene graph.
    pub snapshots: Vec<Snapshot>,
}

/// A snapshot of the scene graph at a specific point in time.
/// It contains nodes and edges representing objects and their relationships.
/// It also includes metadata such as the number of objects and small objects.
#[derive(Serialize, Deserialize, Debug)]
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

/// A node in the scene graph representing an object.
#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    /// Unique identifier for the node.
    #[serde(rename = "node_id")]
    id: usize,

    /// Labels associated with the node.
    label: Vec<String>,

    /// Affordance labels associated with the node, if any.
    label_affordance: Option<Vec<String>>,

    /// Last snapshot index when the node was processed.
    processed_last: usize,

    /// list of features representing the object.
    features: Vec<f32>,

    /// 3D points of the point cloud representing the object.
    pcd_points: Vec<[f32; 3]>,

    /// Colors of the points in the point cloud.
    pcd_colors: Vec<[f32; 3]>,
}

/// An edge in the scene graph representing a relationship between two nodes.
/// The edge can represent both hierarchical and relational relation between two nodes .
#[derive(Serialize, Deserialize, Debug)]
pub struct Edge {
    /// Source node index.
    src: usize,

    /// Destination node index.
    dst: usize,

    /// Metadata associated with the edge.
    meta: EdgeMeta,
}

/// Metadata for an edge in the scene graph.
#[derive(Serialize, Deserialize, Debug)]
struct EdgeMeta {
    /// Type of the edge (e.g., "parent", "on", "next_to").
    desc: String,
}
