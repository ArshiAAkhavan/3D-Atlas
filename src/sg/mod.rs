mod layer;
mod node;
mod sg;

// use layer::Layer;
use node::{Coordinate, Edge, Feature, Node};

pub use layer::Layer;
pub use sg::SceneGraph;

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::Result;

    #[test]
    fn api() -> Result<()> {
        let mut sg = SceneGraph::default();

        // create nodes
        let node1 = sg.new_node(vec![Feature::new("name", "Node 1")]);
        let node2 = sg.new_node(vec![Feature::new("name", "Node 2")]);
        let node3 = sg.new_node(vec![Feature::new("name", "Node 3")]);
        let id2 = node2.id;
        let id3 = node3.id;
        let id1 = node1.id;

        // create a semantic layer and add nodes to layers
        let semantic_layer = sg.new_semantic_layer();
        semantic_layer.push_node(node2);
        semantic_layer.push_node(node3);
        // create the second semantic layer
        let semantic_layer = sg.new_semantic_layer();
        semantic_layer.push_node(node1);

        // nesting
        assert!(sg.nest(id2).under(id1).is_ok());
        assert!(sg.nest(id3).under(id1).is_ok());
        assert_eq!(sg.node(id1)?.children, vec![id2, id3]);
        assert_eq!(sg.node(id2)?.pid, Some(id1));
        assert_eq!(sg.node(id3)?.pid, Some(id1));

        // add edge
        sg.layer_mut(0)?.add_edge(id2, id3, "connected to")?;
        sg.layer_mut(0)?.add_edge(id3, id2, "is supporting")?;
        assert_eq!(sg.node(id2)?.edges.len(), 1);
        assert_eq!(sg.node(id3)?.edges.len(), 1);

        // delete edge
        sg.layer_mut(0)?.del_edge(id2, id3)?;
        assert_eq!(sg.node(id2)?.edges.len(), 0);
        assert_eq!(sg.node(id3)?.edges.len(), 1);

        // delete invalid edge
        assert!(sg.layer_mut(0)?.del_edge(id2, id3).is_err());

        // delete node

        // deleting a node should also delete its edges within the same layers
        sg.del_node(id2)?;
        assert!(sg.node(id2).is_err());
        assert_eq!(sg.node(id3)?.edges.len(), 0);
        // should also be removed from parent's children List
        assert_eq!(sg.node(id1)?.children, vec![id3]);

        sg.del_node(id1)?;
        assert!(sg.node(id1).is_err());
        // children should also be deleted
        assert!(sg.node(id2).is_err());
        assert!(sg.node(id3).is_err());

        Ok(())
    }

    #[test]
    fn query() -> Result<()> {
        let mut sg = SceneGraph::default();

        // create nodes
        let chair = sg.new_node(vec![
            Feature::new("name", "chair"),
            Feature::new("type", "furniture"),
            Feature::new("affordance", "sit"),
        ]);
        let table = sg.new_node(vec![
            Feature::new("name", "table"),
            Feature::new("type", "furniture"),
            Feature::new("affordance", "place items"),
        ]);
        let wall = sg.new_node(vec![
            Feature::new("name", "wall"),
            Feature::new("type", "structure"),
            Feature::new("affordance", "support"),
        ]);
        let clock = sg.new_node(vec![
            Feature::new("name", "clock"),
            Feature::new("type", "appliance"),
        ]);
        let chair_id = chair.id;
        let table_id = table.id;
        let wall_id = wall.id;
        let clock_id = clock.id;

        // create layers and add nodes to layers
        let l = sg.new_semantic_layer();
        l.push_node(table);
        l.push_node(wall);
        l.push_node(chair);
        l.push_node(clock);

        l.add_edge(clock_id, wall_id, "supported by")?;
        l.add_edge(table_id, chair_id, "next to")?;
        l.add_edge(chair_id, table_id, "next to")?;
        l.add_edge(table_id, wall_id, "in front of")?;

        // query nodes by label
        let furniture = sg.nodes_having_features(&["type"]);
        assert_eq!(furniture.len(), 1); // only one layer in the scene graph
        assert_eq!(furniture[0].len(), 4); // all nodes have "type" feature
        let furniture = sg.nodes_matching_features(&[&Feature::new("type", "furniture")]);
        assert_eq!(furniture.len(), 1); // only one layer in the scene graph
        assert_eq!(furniture[0].len(), 2); // only chair and table are furniture 
        assert!(furniture[0].iter().any(|n| n.id == chair_id));
        assert!(furniture[0].iter().any(|n| n.id == table_id));

        // query nodes by affordance
        let sit_nodes = sg.nodes_having_features(&["affordance"]);
        assert_eq!(sit_nodes.len(), 1); // only one layer in the scene graph
        assert_eq!(sit_nodes[0].len(), 3); // chair, table, wall have "affordance" feature
        let sit_nodes = sg.nodes_matching_features(&[&Feature::new("affordance","sit")]);
        assert_eq!(sit_nodes.len(), 1); // only one layer in the scene graph
        assert_eq!(sit_nodes[0].len(), 1); // only chair has "sit" affordance
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
                .any(|e| e.src == table_id && e.dst == chair_id)
        );
        assert!(
            next_to_edges[0]
                .iter()
                .any(|e| e.src == chair_id && e.dst == table_id)
        );

        Ok(())
    }
}
