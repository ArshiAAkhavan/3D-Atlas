use crate::error::{AtlasError, Result};

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub pid: Option<usize>,
    pub children: Vec<usize>,
    pub(super) edges: Vec<Edge>,
    pub features: Vec<Feature>,
    coordinates: Option<Coordinate>,
}

impl Node {
    pub fn new(id: usize, features: Vec<Feature>, coordinates: Option<Coordinate>) -> Self {
        Self {
            id,
            pid: None,
            children: Vec::new(),
            edges: Vec::new(),
            features,
            coordinates,
        }
    }
    pub fn has_feature(&self, key: &str) -> bool {
        self.features.iter().any(|f| f.key == key)
    }

    pub fn match_feature(&self, f: &Feature) -> bool {
        self.features.contains(f)
    }

    fn feature(&self, key: &str) -> Result<&str> {
        self.features
            .iter()
            .find(|f| f.key == key)
            .map(|f| f.value.as_str())
            .ok_or_else(|| AtlasError::FeatureNotFound(key.to_string()))
    }

    pub(super) fn remove_child(&mut self, nid: usize) -> Result<()> {
        let index = self
            .children
            .iter()
            .position(|&id| id == nid)
            .ok_or(AtlasError::NodeNotFound)?;
        self.children.swap_remove(index);
        Ok(())
    }

    pub(super) fn add_child(&mut self, nid: usize) {
        if !self.children.contains(&nid) {
            self.children.push(nid);
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Coordinate {
    x: f32,
    y: f32,
    z: f32,
}
impl Coordinate {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Semantic;

#[derive(Debug, Clone,Eq,PartialEq)]
pub struct Feature {
    key: String,
    value: String,
}
impl Feature {
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub src: usize,
    pub dst: usize,
    pub desc: String,
}

impl Edge {
    pub fn new(src: usize, dst: usize, desc: &str) -> Self {
        Self {
            src,
            dst,
            desc: desc.to_string(),
        }
    }
}
