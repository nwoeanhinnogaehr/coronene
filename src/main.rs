mod player;

use player::graph::{Graph, NodeRef};
use player::board::{Board, Cell};

fn main() {
    let g = Graph::new(NodeRef::new("Hello, world!")).root();
    println!("{}", g.node().data);

    let mut b = Board::new(9, 9);
    b[(0, 0)] = Cell::Black;
    b[(8, 8)] = Cell::Black;
    b[(1, 2)] = Cell::White;
    b[(2, 3)] = Cell::White;
    b[(2, 4)] = Cell::Black;
    b[(3, 3)] = Cell::Black;
    println!("{}", b);
}
