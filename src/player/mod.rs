pub mod graph;
pub mod board;
pub mod htp;

use self::board::{Board, Coord, Color, Move};
use rand::{self, ThreadRng, Rng};

pub trait Player {
    fn generate_move(&mut self, color: Color) -> Move;
    fn play_move(&mut self, m: Move) -> bool;
    fn board(&self) -> &Board;
    fn name(&self) -> String;
    fn version(&self) -> String;
    fn set_board_size(&mut self, cols: Coord, rows: Coord);
}

pub struct RandomPlayer {
    board: Board,
    rng: ThreadRng,
}

impl RandomPlayer {
    pub fn new() -> RandomPlayer {
        RandomPlayer {
            board: Board::new(13, 13),
            rng: rand::thread_rng(),
        }
    }
}

impl Player for RandomPlayer {
    fn generate_move(&mut self, color: Color) -> Move {
        let pos = self.rng.choose(&self.board.empty_cells()).unwrap().clone();
        let m = Move::new(color, pos);
        self.play_move(m);
        m
    }

    fn play_move(&mut self, m: Move) -> bool {
        return self.board.play(m);
    }

    fn board(&self) -> &Board {
        &self.board
    }

    fn name(&self) -> String {
        "coronene".into()
    }
    fn version(&self) -> String {
        "0.000000000000001".into()
    }

    fn set_board_size(&mut self, cols: Coord, rows: Coord) {
        self.board = Board::new(cols, rows);
    }
}
