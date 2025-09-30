use crate::error::{AtlasError, Result};
#[derive(Debug, Default)]
pub struct SceneGraph {
    layers: Vec<SubGraph>,
    node_count: usize,
}

#[derive(Debug, Default)]
struct SubGraph {
    nodes: Vec<(Node, Vec<Edge>)>,
}

#[derive(Debug, Default)]
pub struct Node {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub children_ids: Vec<usize>,
    pub data: Vec<NodeData>,
}

#[derive(Debug, Clone)]
pub enum NodeData {
    Label(String),
    Affordance(String),
    Point { loc: [f32; 3], color: [f32; 3] },
}

#[derive(Debug)]
struct Edge {
    src: usize,
    dst: usize,
    meta: EdgeMeta,
}

#[derive(Debug)]
struct EdgeMeta {
    desc: String,
}
/// public API
impl SceneGraph {
    pub fn add_layer(&mut self) {
        self.layers.push(SubGraph::default());
    }

    pub fn get_layer_mut(&mut self, index: usize) -> Result<&mut SubGraph> {
        let layers_count = self.layers.len();
        self.layers
            .get_mut(index)
            .ok_or(AtlasError::LayerOutOfBounds(index, layers_count))
    }

    pub fn get_layer(&self, index: usize) -> Result<&SubGraph> {
        self.layers
            .get(index)
            .ok_or(AtlasError::LayerOutOfBounds(index, self.layers.len()))
    }

    pub fn get_node(&self, id: usize) -> Result<&Node> {
        self.layers
            .iter()
            .filter_map(|l| l.get_node(id).ok())
            .nth(1)
            .ok_or(AtlasError::NodeNotFound)
    }

    pub fn get_node_mut(&mut self, id: usize) -> Result<&mut Node> {
        self.layers
            .iter_mut()
            .filter_map(|l| l.get_node_mut(id).ok())
            .nth(1)
            .ok_or(AtlasError::NodeNotFound)
    }

    pub fn new_node(&mut self, data: Vec<NodeData>) -> Node {
        self.node_count += 1;
        Node {
            id: self.node_count - 1,
            parent_id: None,
            children_ids: vec![],
            data,
        }
    }

    pub fn del_node(&mut self, id: usize) -> Result<Node> {
        let layer_id = self
            .layers
            .iter()
            .position(|g| g.nodes.iter().any(|(n, _)| n.id == id))
            .ok_or(AtlasError::NodeNotFound)?;

        let node = self.del_node_on_layer(id, layer_id)?;

        if let Some(parent_id) = node.parent_id {
            let parent = self.get_layer_mut(layer_id - 1)?.get_node_mut(parent_id)?;
            let pos = parent.children_ids.iter().position(|e| *e == id).unwrap();
            parent.children_ids.swap_remove(pos);
        }
        Ok(node)
    }

    pub fn nest(&mut self, nestee: usize) -> NestUnder<'_> {
        NestUnder { sg: self, nestee }
    }
}

impl SceneGraph {
    fn total_nodes(&self) -> usize {
        self.layers.iter().map(|g| g.nodes.len()).sum()
    }

    fn del_node_on_layer(&mut self, id: usize, layer_id: usize) -> Result<Node> {
        let node = self.get_layer_mut(layer_id)?.del_node(id)?;
        for child_id in &node.children_ids {
            self.del_node_on_layer(*child_id, layer_id + 1)?;
        }
        Ok(node)
    }
}

impl SubGraph {
    pub fn get_node(&self, id: usize) -> Result<&Node> {
        self.nodes
            .iter()
            .map(|(n, _)| n)
            .find(|n| n.id == id)
            .ok_or(AtlasError::NodeNotFound)
    }

    pub fn get_node_mut(&mut self, id: usize) -> Result<&mut Node> {
        self.nodes
            .iter_mut()
            .map(|(n, _)| n)
            .find(|n| n.id == id)
            .ok_or(AtlasError::NodeNotFound)
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push((node, Vec::new()));
    }

    fn del_node(&mut self, id: usize) -> Result<Node> {
        let pos = self
            .nodes
            .iter()
            .position(|(n, _)| n.id == id)
            .ok_or(AtlasError::NodeNotFound)?;
        let (node, _) = self.nodes.swap_remove(pos);
        for (_, edges) in self.nodes.iter_mut() {
            edges.retain(|e| e.dst != id);
        }
        Ok(node)
    }

    pub fn add_edge(&mut self, src: usize, dst: usize, meta: EdgeMeta) -> Result<()> {
        let _ = self
            .nodes
            .iter()
            .find(|(n, _)| n.id == dst)
            .ok_or(AtlasError::NodeNotFound)?;
        let (_, src_edges) = self
            .nodes
            .iter_mut()
            .find(|(n, _)| n.id == src)
            .ok_or(AtlasError::NodeNotFound)?;
        src_edges.push(Edge { src, dst, meta });
        Ok(())
    }

    pub fn del_edge(&mut self, src: usize, dst: usize) -> Result<()> {
        let (_, src_edges) = self
            .nodes
            .iter_mut()
            .find(|(n, _)| n.id == src)
            .ok_or(AtlasError::NodeNotFound)?;
        let edge_pos = src_edges
            .iter()
            .position(|e| e.dst == dst)
            .ok_or(AtlasError::EdgeNotFound)?;
        src_edges.swap_remove(edge_pos);
        Ok(())
    }
}

struct NestUnder<'a> {
    sg: &'a mut SceneGraph,
    nestee: usize,
}

impl<'a> NestUnder<'a> {
    pub fn under(&mut self, nester: usize) -> Result<&mut SceneGraph> {
        let nestee = self.sg.get_node_mut(self.nestee)?;
        match nestee.parent_id {
            Some(parent_id) => {
                nestee.parent_id = Some(nester);
                let parent = self.sg.get_node_mut(parent_id)?;
                if let Some(pos) = parent.children_ids.iter().position(|c| *c == self.nestee) {
                    parent.children_ids.swap_remove(pos);
                }
            }
            None => nestee.parent_id = Some(nester),
        }

        self.sg.get_node_mut(nester)?.children_ids.push(self.nestee);
        Ok(self.sg)
    }
}
