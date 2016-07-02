mod player;

use player::graph::{Graph, NodeRef};

fn main() {
    let g = Graph::new(NodeRef::new("Hello, world!")).root();
    println!("{}", g.node().data);
}
