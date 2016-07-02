mod player;

use player::graph::{NodeRef};
use player::board::{Board, Color, Move};

fn main() {
    let mut g = NodeRef::new_root();
    for i in 0..13 {
        g = g.add_child(NodeRef::new(Move::new(Color::Black, (i, i))));
    }

    let mut b = Board::new(13, 13);
    g.fill_board(&mut b);
    println!("{}", b);
}
