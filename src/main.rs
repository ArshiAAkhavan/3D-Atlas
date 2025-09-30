// use std::collections::HashSet;
// use std::fs::File;
// use std::io::BufReader;
//
// use atlas::SceneGraph;
fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // let snapshot_path = "./data/scenegraph_scena_unity_pretty.json";
//     let snapshot_path = "./data/map1_conf_1_pretty.json";
//     let file = File::open(snapshot_path)?;
//     let reader = BufReader::new(file);
//     let snapshot: SceneGraph = serde_json::from_reader(reader)?;
//     println!("node count: {}", snapshot.nodes.len());
//     println!("edge count: {}", snapshot.edges.len());
//     println!("expected node count: {}", snapshot.num_objects);
//     println!("small objects: {:?}", snapshot.small_objects);
//
//     let edge_types = snapshot
//         .edges
//         .iter()
//         .map(|e| e.meta.desc.clone())
//         .collect::<HashSet<_>>();
//     println!("edge types: {edge_types:?}");
//     // snapshot.nodes.iter().map(|n|n.pcd_points.len()).for_each(|c|println!("{c}"));
//     let ids = snapshot
//         .edges
//         .iter()
//         .flat_map(|e| vec![e.src, e.dst])
//         .collect::<HashSet<_>>();
//     let mut ids = ids.iter().collect::<Vec<_>>();
//     ids.sort();
//     ids.into_iter().for_each(|i| println!("{i}"));
//
    Ok(())
}
