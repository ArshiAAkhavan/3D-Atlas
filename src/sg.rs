use crate::error::{AtlasError, Result};

/// A hierarchical representation of objects and their relationships in a 3D environment.
/// The scene graph is organized into layers, where each layer contains nodes representing objects.
/// Nodes on adjacent layers can have parent-child relationships, and edges can represent various
/// types of connections between nodes.
///
/// The scene graph supports operations such as adding/removing nodes and edges,
/// nesting nodes under other nodes, and querying nodes by their IDs.
#[derive(Debug, Default)]
pub struct SceneGraph {
    /// Layers of the scene graph, where each layer is a subgraph.
    layers: Vec<SubGraph>,

    /// Total number of nodes in the scene graph.
    /// This is used to assign unique IDs to new nodes.
    node_count: usize,
}

/// A subgraph representing a single layer in the scene graph.
/// Each layer contains nodes and edges connecting those nodes.
/// Nodes in a layer can have parent-child relationships with nodes in adjacent layers.
/// Edges can represent various types of connections between nodes.
/// For example, a layer might represent a specific level of detail or a specific type of object in the scene.
/// Edges can represent relationships such as "is part of", "is connected to", or "is near".
#[derive(Debug, Default)]
pub struct SubGraph {
    /// Nodes in the subgraph, each paired with its outgoing edges.
    nodes: Vec<(Node, Vec<Edge>)>,
}

/// A node in the scene graph representing an object or entity in the 3D environment.
/// Each node has a unique ID, optional parent ID, a list of child IDs, and associated data.
/// The data can include labels, affordances, and point cloud information.
#[derive(Debug, Default)]
pub struct Node {
    /// Unique identifier for the node in the scene graph.
    pub id: usize,

    /// Optional identifier of the parent node, if any.
    /// It is assured that if a node has a parent, the parent is on the layer immediately above
    /// (Higher Layer ID).
    pub parent_id: Option<usize>,

    /// List of identifiers for child nodes.
    /// It is assured that if a node has children, they are on the layer immediately below
    pub children_ids: Vec<usize>,

    /// Data associated with the node, such as labels, affordances, and point cloud information.
    pub data: Vec<NodeData>,
}

/// Different types of data that can be associated with a node in the scene graph.
/// This can include labels, affordances, and point cloud information.
#[derive(Debug, Clone)]
pub enum NodeData {
    /// A textual label describing the object or entity represented by the node.
    Label(String),

    /// A textual description of the affordance or functionality of the object.
    Affordance(String),

    /// A point in 3D space with a location and color information.
    Point { loc: [f32; 3], color: [f32; 3] },
}

/// An edge in the scene graph representing a connection or relationship between two nodes.
/// Each edge has a source node ID, a destination node ID, and associated metadata describing the
/// nature of the connection.
/// Edges can represent various types of relationships, such as "is part of", "is connected to",
/// or "is near".
/// Edges are directed, meaning they have a specific direction from the source node to the destination node.
/// Edges can only connect nodes within the same layer.
#[derive(Debug)]
struct Edge {
    /// Identifier of the source node.
    src: usize,

    /// Identifier of the destination node.
    dst: usize,

    /// Metadata describing the nature of the connection.
    meta: EdgeMeta,
}

/// Metadata associated with an edge in the scene graph.
#[derive(Debug)]
struct EdgeMeta {
    /// A textual description of the edge, such as the type of relationship it represents.
    desc: String,
}

/// public API
impl SceneGraph {
    /// Create a new, empty layer in the scene graph.
    pub fn create_layer(&mut self) {
        self.layers.push(SubGraph::default());
    }

    /// Get a mutable reference to a layer by its index.
    pub fn get_layer_mut(&mut self, index: usize) -> Result<&mut SubGraph> {
        let layers_count = self.layers.len();
        self.layers
            .get_mut(index)
            .ok_or(AtlasError::LayerOutOfBounds(index, layers_count))
    }

    /// Get an immutable reference to a layer by its index.
    pub fn get_layer(&self, index: usize) -> Result<&SubGraph> {
        self.layers
            .get(index)
            .ok_or(AtlasError::LayerOutOfBounds(index, self.layers.len()))
    }

    /// Get a reference to a node by its ID.
    /// Searches through all layers.
    pub fn get_node(&self, id: usize) -> Result<&Node> {
        self.layers
            .iter()
            .filter_map(|l| l.get_node(id).ok())
            .nth(0)
            .ok_or(AtlasError::NodeNotFound)
    }

    /// Get a mutable reference to a node by its ID.
    /// Searches through all layers.
    pub fn get_node_mut(&mut self, id: usize) -> Result<&mut Node> {
        self.layers
            .iter_mut()
            .filter_map(|l| l.get_node_mut(id).ok())
            .nth(0)
            .ok_or(AtlasError::NodeNotFound)
    }

    /// Create a new node with the given data for the Scene Graph.
    pub fn new_node(&mut self, data: Vec<NodeData>) -> Node {
        self.node_count += 1;
        Node {
            id: self.node_count - 1,
            parent_id: None,
            children_ids: vec![],
            data,
        }
    }

    /// Delete a node by its ID from the Scene Graph.
    /// This will also recursively delete all child nodes of the specified node.
    /// If the node has a parent, it will be removed from the parent's list of children.
    pub fn del_node(&mut self, id: usize) -> Result<Node> {
        // Find the layer containing the node
        let layer_id = self
            .layers
            .iter()
            .position(|g| g.nodes.iter().any(|(n, _)| n.id == id))
            .ok_or(AtlasError::NodeNotFound)?;

        let node = self.del_node_on_layer(id, layer_id)?;

        // If the node has a parent, remove it from the parent's children list
        if let Some(parent_id) = node.parent_id {
            let parent = self.get_layer_mut(layer_id - 1)?.get_node_mut(parent_id)?;
            let pos = parent.children_ids.iter().position(|e| *e == id).unwrap();
            parent.children_ids.swap_remove(pos);
        }
        Ok(node)
    }

    /// Nest a node under another node, establishing a parent-child relationship.
    /// The `nestee` node will become a child of the `nester` node.
    /// Both nodes must exist in the scene graph.
    /// If the `nestee` node already has a parent, it will be removed from its current parent's list of children.
    /// The `nester` node will have the `nestee` node added to its list of children.
    ///
    /// ```rust
    /// # use atlas::SceneGraph;
    /// # use atlas::NodeData;
    /// # let mut sg = SceneGraph::default();
    ///
    /// // Create nodes
    /// let node1 = sg.new_node(vec![NodeData::Label("Node 1".to_string())]);
    /// let node2 = sg.new_node(vec![NodeData::Label("Node 2".to_string())]);
    /// let id1 = node1.id;
    /// let id2 = node2.id;
    ///
    /// // Create a layer and add nodes to it
    /// sg.create_layer();
    /// sg.get_layer_mut(0).unwrap().add_node(node1);
    /// sg.create_layer();
    /// sg.get_layer_mut(1).unwrap().add_node(node2);
    ///
    /// // Nest node2 under node1
    /// sg.nest(id2).under(id1).unwrap();
    ///
    /// assert_eq!(sg.get_node(id1).unwrap().children_ids, vec![id2]);
    /// assert_eq!(sg.get_node(id2).unwrap().parent_id, Some(id1));
    /// ```
    pub fn nest(&mut self, nestee: usize) -> NestUnder<'_> {
        NestUnder { sg: self, nestee }
    }
}

impl SceneGraph {

    /// Delete a node by its ID from a specific layer in the Scene Graph.
    /// This will also recursively delete all child nodes of the specified node from subsequent layers.
    /// If the node has a parent, it WILL NOT be removed from the parent's list of children
    fn del_node_on_layer(&mut self, id: usize, layer_id: usize) -> Result<Node> {
        let node = self.get_layer_mut(layer_id)?.del_node(id)?;
        for child_id in &node.children_ids {
            self.del_node_on_layer(*child_id, layer_id + 1)?;
        }
        Ok(node)
    }
}

impl SubGraph {
    /// Get a reference to a node by its ID within the subgraph.
    pub fn get_node(&self, id: usize) -> Result<&Node> {
        self.nodes
            .iter()
            .map(|(n, _)| n)
            .find(|n| n.id == id)
            .ok_or(AtlasError::NodeNotFound)
    }

    /// Get a mutable reference to a node by its ID within the subgraph.
    pub fn get_node_mut(&mut self, id: usize) -> Result<&mut Node> {
        self.nodes
            .iter_mut()
            .map(|(n, _)| n)
            .find(|n| n.id == id)
            .ok_or(AtlasError::NodeNotFound)
    }

    /// Add a new node to the subgraph.
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push((node, Vec::new()));
    }

    /// Delete a node by its ID from the subgraph.
    /// This will also remove all edges connected to the node, both incoming and outgoing.
    fn del_node(&mut self, id: usize) -> Result<Node> {
        let pos = self
            .nodes
            .iter()
            .position(|(n, _)| n.id == id)
            .ok_or(AtlasError::NodeNotFound)?;
        let (node, _) = self.nodes.swap_remove(pos);

        // Remove all edges connected to the node
        for (_, edges) in self.nodes.iter_mut() {
            edges.retain(|e| e.dst != id);
        }
        Ok(node)
    }

    /// Add a directed edge from the source node to the destination node with associated metadata.
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

    /// Delete a directed edge from the source node to the destination node.
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

pub struct NestUnder<'a> {
    sg: &'a mut SceneGraph,
    nestee: usize,
}

impl<'a> NestUnder<'a> {
    /// Complete the nesting operation by specifying the `nester` node under which the `nestee` node
    /// Refer to the `nest` method in `SceneGraph` for usage example.
    ///
    /// [`nest`](SceneGraph::nest)
    pub fn under(&mut self, nester: usize) -> Result<&mut SceneGraph> {
        let nestee = self.sg.get_node_mut(self.nestee)?;
        match nestee.parent_id {
            // Remove from old parent
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
