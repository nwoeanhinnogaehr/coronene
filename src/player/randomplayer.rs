use super::Player;
use super::board::{Board, Move, Color, Coord, Pos};
use rand::{self, ThreadRng, Rng};

pub struct RandomPlayer {
    board: Board,
    rng: ThreadRng,
    moves: Vec<Move>,
}

impl RandomPlayer {
    pub fn new() -> RandomPlayer {
        RandomPlayer {
            board: Board::new((13, 13)),
            rng: rand::thread_rng(),
            moves: Vec::new(),
        }
    }
}

impl Player for RandomPlayer {
    fn generate_move(&mut self, color: Color) -> Move {
        if self.board.check_win().is_some() {
            return Move::Resign;
        }
        let empty_cells: Vec<Pos> = self.board.empty_cells().collect();
        let pos = self.rng.choose(&empty_cells);
        let m = match pos {
            Some(pos) => Move::new(color, *pos),
            None => Move::Resign,
        };
        self.play_move(m);
        m
    }

    fn play_move(&mut self, m: Move) -> bool {
        self.moves.push(m);
        return self.board.play(m);
    }

    fn undo(&mut self) {
        if let Some(m) = self.moves.pop() {
            m.pos().map(|pos| self.board.clear_cell(pos));
        }
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
        self.board = Board::new((cols, rows));
    }
}
