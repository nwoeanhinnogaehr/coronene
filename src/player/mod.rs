macro_rules! eprintln(
    ($($arg:tt)*) => { {
        use std::io::Write;
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

pub mod graph;
pub mod board;
pub mod htp;
//pub mod randomplayer;
pub mod mctsplayer;
pub mod misc;

use self::board::{Board, Coord, Color, Move};

pub trait Player {
    fn generate_move(&mut self, color: Color) -> Move;
    fn play_move(&mut self, m: Move) -> bool;
    fn undo(&mut self);
    fn board(&self) -> &Board;
    fn name(&self) -> String;
    fn version(&self) -> String;
    fn set_board_size(&mut self, cols: Coord, rows: Coord);
}
