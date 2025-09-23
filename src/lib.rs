use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Snapshot {
    small_objects: Vec<usize>,
    num_objects: usize,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Node{
    id: usize,
    label: Vec<String>,
    label_affordance: Option<Vec<String>>,
    processed_last: usize,
    features: Vec<f32>,
    pcd_points: Vec<[f32; 3]>,
    pcd_colors: Vec<[u8; 3]>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Edge{
    src: usize,
    dst: usize,
    meta: EdgeMeta,
}

#[derive(Serialize, Deserialize, Debug)]
struct EdgeMeta{
    desc: String,
}
