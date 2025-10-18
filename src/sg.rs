use crate::error::{AtlasError, Result};
use crate::layer::{Edge, EdgeView, Feature, Layer, Node};

/// A hierarchical representation of objects and their relationships in a 3D environment.
/// The scene graph is organized into layers, where each layer contains nodes representing objects.
/// Nodes on adjacent layers can have parent-child relationships, and edges can represent various
/// types of connections between nodes.
///
/// The scene graph supports operations such as adding/removing nodes and edges,
/// nesting nodes under other nodes, and querying nodes by their IDs.
#[derive(Debug, Default, Clone)]
pub struct SceneGraph {
    /// Layers of the scene graph, where each layer is a layer.
    layers: Vec<Layer>,

    /// Total number of nodes in the scene graph.
    /// This is used to assign unique IDs to new nodes.
    node_count: usize,
}

/// public API
impl SceneGraph {
    /// Create a new, empty layer in the scene graph.
    pub fn create_layer(&mut self) {
        self.layers.push(Default::default());
    }

    pub fn top_layer_mut(&mut self) -> Result<&mut Layer> {
        self.layers
            .last_mut()
            .ok_or(AtlasError::LayerOutOfBounds(0, 0))
    }

    pub fn top_layer(&self) -> Result<&Layer> {
        self.layers.last().ok_or(AtlasError::LayerOutOfBounds(0, 0))
    }

    /// Get a mutable reference to a layer by its index.
    pub fn get_layer_mut(&mut self, index: usize) -> Result<&mut Layer> {
        let layers_count = self.layers.len();
        self.layers
            .get_mut(index)
            .ok_or(AtlasError::LayerOutOfBounds(index, layers_count))
    }

    /// Get an immutable reference to a layer by its index.
    pub fn get_layer(&self, index: usize) -> Result<&Layer> {
        self.layers
            .get(index)
            .ok_or(AtlasError::LayerOutOfBounds(index, self.layers.len()))
    }
}

/// Query methods for retrieving nodes and edges
impl SceneGraph {
    /// Get a reference to a node by its ID.
    /// Searches through all layers.
    pub fn get_node(&self, id: usize) -> Result<&Node> {
        self.layers
            .iter()
            .filter_map(|l| l.node(id).ok())
            .nth(0)
            .ok_or(AtlasError::NodeNotFound)
    }

    /// Get a mutable reference to a node by its ID.
    /// Searches through all layers.
    pub fn get_node_mut(&mut self, id: usize) -> Result<&mut Node> {
        self.layers
            .iter_mut()
            .filter_map(|l| l.node_mut(id).ok())
            .nth(0)
            .ok_or(AtlasError::NodeNotFound)
    }

    /// Get List of all nodes matching a specific node features.
    pub fn nodes_with_features(&self, features: &[Feature]) -> Vec<Vec<&Node>> {
        self.layers
            .iter()
            .map(|l| l.nodes_with_features(features))
            .collect()
    }

    /// Get List of all edges matching a specific description.
    pub fn edges_matching(&self, desc: &str) -> Vec<Vec<EdgeView>> {
        self.layers.iter().map(|l| l.edges_matching(desc)).collect()
    }

    /// Get List of all edges from a specific source node.
    pub fn edges_from(&self, src: usize) -> Vec<&Edge> {
        self.layers
            .iter()
            .map(|l| l.edges_from(src))
            .filter(|v| !v.is_empty())
            .nth(0)
            .unwrap_or_default()
    }

    /// Get List of all edges to a specific destination node.
    pub fn edges_to(&self, dst: usize) -> Vec<EdgeView> {
        self.layers
            .iter()
            .filter(|l| l.node(dst).is_ok())
            .nth(0)
            .map(|l| l.edges_to(dst))
            .unwrap_or_default()
    }

    /// Get a reference to a node's edges by its ID.
    #[cfg(test)]
    fn get_edges(&self, id: usize) -> Result<&Vec<Edge>> {
        self.layers
            .iter()
            .filter_map(|l| l.node(id).ok())
            .nth(0)
            .map(|n| &n.edges)
            .ok_or(AtlasError::NodeNotFound)
    }
}

/// API
impl SceneGraph {
    /// Create a new node with the given data for the Scene Graph.
    pub fn new_node(&mut self, features: Vec<Feature>) -> Node {
        self.node_count += 1;
        Node {
            id: self.node_count - 1,
            parent_id: None,
            children_ids: vec![],
            features,
            edges: vec![],
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
            .position(|g| g.nodes.iter().any(|n| n.id == id))
            .ok_or(AtlasError::NodeNotFound)?;

        let node = self.del_node_on_layer(id, layer_id)?;

        // If the node has a parent, remove it from the parent's children list
        if let Some(parent_id) = node.parent_id {
            let parent = self.get_layer_mut(layer_id + 1)?.node_mut(parent_id)?;
            let pos = parent.children_ids.iter().position(|e| *e == id).unwrap();
            parent.children_ids.swap_remove(pos);
        }
        Ok(node)
    }

    /// Nest a node under another node, establishing a parent-child relationship.
    /// The `nestee` node will become a child of the `nester` node.
    /// Both nodes must exist in the scene graph.
    /// The `nester` node must be on the layer immediately above the `nestee` node.
    /// If the `nestee` node already has a parent, it will be removed from its current parent's list of children.
    /// The `nester` node will have the `nestee` node added to its list of children.
    ///
    /// ```rust
    /// # use atlas::SceneGraph;
    /// # use atlas::Feature;
    /// # let mut sg = SceneGraph::default();
    ///
    /// // Create nodes
    /// let node1 = sg.new_node(vec![Feature::semantic("name","Node 1")]);
    /// let node2 = sg.new_node(vec![Feature::semantic("name","Node 2")]);
    /// let id1 = node1.id;
    /// let id2 = node2.id;
    ///
    /// // Create a layer and add nodes to it
    /// sg.create_layer();
    /// sg.create_layer();
    /// sg.get_layer_mut(0).unwrap().add_node(node1);
    /// sg.get_layer_mut(1).unwrap().add_node(node2);
    ///
    /// // Nest node1 under node2
    /// sg.nest(id1).under(id2).unwrap();
    ///
    /// assert_eq!(sg.get_node(id2).unwrap().children_ids, vec![id1]);
    /// assert_eq!(sg.get_node(id1).unwrap().parent_id, Some(id2));
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
            self.del_node_on_layer(*child_id, layer_id - 1)?;
        }
        Ok(node)
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
        let nester_layer_id = self
            .sg
            .layers
            .iter()
            .position(|g| g.nodes.iter().any(|n| n.id == nester))
            .ok_or(AtlasError::NodeNotFound)?;
        let nestee_layer_id = self
            .sg
            .layers
            .iter()
            .position(|g| g.nodes.iter().any(|n| n.id == self.nestee))
            .ok_or(AtlasError::NodeNotFound)?;

        if nester_layer_id - 1 != nestee_layer_id {
            return Err(AtlasError::InvalidLayersForNesting(
                nestee_layer_id,
                nester_layer_id,
            ));
        }

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::Result;
    use crate::layer::EdgeMeta;

    #[test]
    fn api() -> Result<()> {
        let mut sg = SceneGraph::default();

        // create nodes
        let node1 = sg.new_node(vec![Feature::semantic("name", "Node 1")]);
        let node2 = sg.new_node(vec![Feature::semantic("name", "Node 2")]);
        let node3 = sg.new_node(vec![Feature::semantic("name", "Node 3")]);
        let id1 = node1.id;
        let id2 = node2.id;
        let id3 = node3.id;

        // create layers and add nodes to layers
        sg.create_layer();
        sg.top_layer_mut()?.add_node(node2);
        sg.top_layer_mut()?.add_node(node3);
        sg.create_layer();
        sg.top_layer_mut()?.add_node(node1);

        // nesting
        assert!(sg.nest(id2).under(id1).is_ok());
        assert!(sg.nest(id3).under(id1).is_ok());
        assert_eq!(sg.get_node(id1)?.children_ids, vec![id2, id3]);
        assert_eq!(sg.get_node(id2)?.parent_id, Some(id1));
        assert_eq!(sg.get_node(id3)?.parent_id, Some(id1));

        // add edge
        let meta1 = EdgeMeta {
            desc: "connected to".to_string(),
        };
        let meta2 = EdgeMeta {
            desc: "is supporting".to_string(),
        };
        sg.get_layer_mut(0)?.add_edge(id2, id3, meta1)?;
        sg.get_layer_mut(0)?.add_edge(id3, id2, meta2)?;
        assert_eq!(sg.get_edges(id2)?.len(), 1);
        assert_eq!(sg.get_edges(id3)?.len(), 1);

        // delete edge
        sg.get_layer_mut(0)?.del_edge(id2, id3)?;
        assert_eq!(sg.get_edges(id2)?.len(), 0);
        assert_eq!(sg.get_edges(id3)?.len(), 1);

        // delete invalid edge
        assert!(sg.get_layer_mut(0)?.del_edge(id2, id3).is_err());

        // delete node

        // deleting a node should also delete its edges within the same layers
        sg.del_node(id2)?;
        assert!(sg.get_node(id2).is_err());
        assert_eq!(sg.get_edges(id3)?.len(), 0);
        // should also be removed from parent's children List
        assert_eq!(sg.get_node(id1)?.children_ids, vec![id3]);

        sg.del_node(id1)?;
        assert!(sg.get_node(id1).is_err());
        // children should also be deleted
        assert!(sg.get_node(id2).is_err());
        assert!(sg.get_node(id3).is_err());

        Ok(())
    }

    #[test]
    fn query() -> Result<()> {
        let mut sg = SceneGraph::default();

        // create nodes
        let chair = sg.new_node(vec![
            Feature::semantic("name", "chair"),
            Feature::semantic("type", "furniture"),
            Feature::semantic("affordance", "sit"),
        ]);
        let table = sg.new_node(vec![
            Feature::semantic("name", "table"),
            Feature::semantic("type", "furniture"),
            Feature::semantic("affordance", "place items"),
        ]);
        let wall = sg.new_node(vec![
            Feature::semantic("name", "wall"),
            Feature::semantic("type", "structure"),
            Feature::semantic("affordance", "support"),
        ]);
        let clock = sg.new_node(vec![
            Feature::semantic("name", "clock"),
            Feature::semantic("type", "appliance"),
        ]);
        let chair_id = chair.id;
        let table_id = table.id;
        let wall_id = wall.id;
        let clock_id = clock.id;

        // create layers and add nodes to layers
        sg.create_layer();
        sg.top_layer_mut()?.add_node(table);
        sg.top_layer_mut()?.add_node(wall);
        sg.top_layer_mut()?.add_node(chair);
        sg.top_layer_mut()?.add_node(clock);

        sg.top_layer_mut()?.add_edge(
            clock_id,
            wall_id,
            EdgeMeta {
                desc: "supported by".into(),
            },
        )?;
        sg.top_layer_mut()?.add_edge(
            table_id,
            chair_id,
            EdgeMeta {
                desc: "next to".into(),
            },
        )?;
        sg.top_layer_mut()?.add_edge(
            chair_id,
            table_id,
            EdgeMeta {
                desc: "next to".into(),
            },
        )?;
        sg.top_layer_mut()?.add_edge(
            table_id,
            wall_id,
            EdgeMeta {
                desc: "in front of".into(),
            },
        )?;

        // query nodes by label
        let furniture = sg.nodes_with_features(&[Feature::semantic("type", "furniture")]);
        assert_eq!(furniture.len(), 1); // only one layer in the scene graph
        assert_eq!(furniture[0].len(), 2); // top layer
        assert!(furniture[0].iter().any(|n| n.id == chair_id));
        assert!(furniture[0].iter().any(|n| n.id == table_id));

        // query nodes by affordance
        let sit_nodes = sg.nodes_with_features(&[Feature::semantic("affordance", "sit")]);
        assert_eq!(sit_nodes.len(), 1); // only one layer in the scene graph
        assert_eq!(sit_nodes[0].len(), 1); // top layer
        assert_eq!(sit_nodes[0][0].id, chair_id);

        // query edges by src
        let edges_from_table = sg.edges_from(table_id);
        assert_eq!(edges_from_table.len(), 2);
        assert!(edges_from_table.iter().any(|e| e.dst == chair_id));
        assert!(edges_from_table.iter().any(|e| e.dst == wall_id));

        // query edges by dst;
        let edges_to_wall = sg.edges_to(wall_id);
        assert_eq!(edges_to_wall.len(), 2);
        assert!(edges_to_wall.iter().any(|e| e.src == clock_id));
        assert!(edges_to_wall.iter().any(|e| e.src == table_id));

        // query edges by description
        let next_to_edges = sg.edges_matching("next to");
        assert_eq!(next_to_edges.len(), 1); // only one layer in the scene graph
        assert_eq!(next_to_edges[0].len(), 2); // top layer 
        assert!(
            next_to_edges[0]
                .iter()
                .any(|e| e.src == table_id && e.edge.dst == chair_id)
        );
        assert!(
            next_to_edges[0]
                .iter()
                .any(|e| e.src == chair_id && e.edge.dst == table_id)
        );

        Ok(())
    }
}
