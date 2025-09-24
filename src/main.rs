use std::fs::File;
use std::io::BufReader;

use atlas::Snapshot;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let snapshot_path = "./data/scenegraph_scena_unity_pretty.json";
    let snapshot_path = "./data/map1_conf_1_pretty.json";
    let file = File::open(snapshot_path)?;
    let reader = BufReader::new(file);
    let snapshot: Snapshot = serde_json::from_reader(reader)?;
    println!("node count: {}", snapshot.nodes.len());
    println!("edge count: {}", snapshot.edges.len());
    println!("expected node count: {}", snapshot.num_objects);
    println!("small objects: {:?}", snapshot.small_objects);
    Ok(())
}
