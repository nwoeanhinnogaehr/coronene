use std::ops::{Index, IndexMut};
use super::graph::NodeRef;
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Color {
    Black,
    White,
}

impl FromStr for Color {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().chars().next() {
            Some('b') => Ok(Color::Black),
            Some('w') => Ok(Color::White),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Color::Black => write!(f, "B"),
            Color::White => write!(f, "W"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Black,
    White,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Cell::Empty => write!(f, "+"),
            Cell::Black => write!(f, "B"),
            Cell::White => write!(f, "W"),
        }
    }
}

impl From<Color> for Cell {
    fn from(c: Color) -> Cell {
        match c {
            Color::Black => Cell::Black,
            Color::White => Cell::White,
        }
    }
}

pub type Coord = u8;

#[derive(Copy, Clone)]
pub struct Pos {
    x: Coord,
    y: Coord,
}

impl Pos {
    pub fn new(x: Coord, y: Coord) -> Pos {
        Pos { x: x, y: y }
    }
}

impl FromStr for Pos {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let b = s.as_bytes();
        let x = if b[0] >= b'a' && b[0] <= b'z' {
            b[0] - b'a'
        } else {
            return Err(());
        };
        let y = match s[1..].parse::<Coord>() {
            Ok(y) => y - 1,
            Err(_) => return Err(()),
        };
        Ok(Pos::new(x, y))
    }
}

impl From<(Coord, Coord)> for Pos {
    fn from(p: (Coord, Coord)) -> Pos {
        Pos::new(p.0, p.1)
    }
}

impl<'a> From<&'a str> for Pos {
    fn from(s: &'a str) -> Pos {
        s.parse().expect("position parse failed")
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", (self.x + 'A' as Coord) as char, self.y + 1)
    }
}

#[derive(Copy, Clone)]
pub enum Move {
    Resign,
    None,
    Play {
        color: Color,
        pos: Pos,
    },
}

impl Move {
    pub fn new<P>(color: Color, pos: P) -> Move
        where P: Into<Pos>
    {
        Move::Play {
            color: color,
            pos: pos.into(),
        }
    }

    pub fn pos(&self) -> Option<Pos> {
        if let &Move::Play { color: _, pos } = self {
            Some(pos)
        } else {
            None
        }
    }

    pub fn color(&self) -> Option<Color> {
        if let &Move::Play { color, pos: _ } = self {
            Some(color)
        } else {
            None
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Move::Resign => write!(f, "resign"),
            &Move::None => write!(f, "pass"),
            &Move::Play { color: _, pos } => write!(f, "{}", pos),
        }
    }
}


#[derive(Clone)]
pub struct Board {
    cols: Coord,
    rows: Coord,
    board: Vec<Cell>,
}

impl Board {
    pub fn new(cols: Coord, rows: Coord) -> Board {
        Board {
            cols: cols,
            rows: rows,
            board: vec![Cell::Empty; rows as usize * cols as usize],
        }
    }

    pub fn cols(&self) -> Coord {
        self.cols
    }

    pub fn rows(&self) -> Coord {
        self.rows
    }

    pub fn is_empty<P>(&self, pos: P) -> bool
        where P: Into<Pos>
    {
        self[pos] == Cell::Empty
    }

    pub fn clear_cell<P>(&mut self, pos: P)
        where P: Into<Pos>
    {
        self[pos] = Cell::Empty;
    }

    pub fn play(&mut self, m: Move) -> bool {
        match m {
            Move::Resign | Move::None => true,
            Move::Play { color, pos } => {
                if !self.is_empty(pos) {
                    false
                } else {
                    self[pos] = color.into();
                    true
                }
            }
        }
    }

    pub fn empty_cells(&self) -> Vec<Pos> {
        let mut cells = Vec::new();
        for x in 0..self.cols() {
            for y in 0..self.rows() {
                if self.is_empty((x, y)) {
                    cells.push((x, y).into());
                }
            }
        }
        cells
    }

    fn idx_of(&self, pos: Pos) -> Option<usize> {
        if pos.x >= self.cols || pos.y >= self.rows {
            None
        } else {
            Some(pos.y as usize * self.rows as usize + pos.x as usize)
        }
    }
}

impl<T> Index<T> for Board
    where T: Into<Pos>
{
    type Output = Cell;

    fn index(&self, idx: T) -> &Self::Output {
        let idx = idx.into();
        let idx = self.idx_of(idx).expect("board index out of bounds");
        &self.board[idx]
    }
}

impl<T> IndexMut<T> for Board
    where T: Into<Pos>
{
    fn index_mut(&mut self, idx: T) -> &mut Self::Output {
        let idx = idx.into();
        let idx = self.idx_of(idx).expect("board index out of bounds");
        &mut self.board[idx]
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "  "));
        for x in 0..self.cols {
            try!(write!(f, "{} ", (x + 'a' as Coord) as char));
        }
        for y in 0..self.rows {
            try!(write!(f, "\n"));
            for _ in 0..y {
                try!(write!(f, " "));
            }
            try!(write!(f, "{:2}\\", y + 1));
            for x in 0..self.cols - 1 {
                try!(write!(f, "{} ", self[(x, y)]));
            }
            try!(write!(f, "{}\\{}", self[(self.cols - 1, y)], y + 1));
        }
        try!(write!(f, "\n   "));
        for _ in 0..self.rows {
            try!(write!(f, " "));
        }
        for x in 0..self.cols {
            try!(write!(f, "{} ", (x + 'a' as Coord) as char));
        }
        Ok(())
    }
}

impl<T> NodeRef<Option<T>>
    where T: Into<Move> + Clone
{
    /// Given a node and a board, follow the first parent of each node up to the root, filling in
    /// the move for each node on the board.
    pub fn fill_board(&self, board: &mut Board) {
        let node = self.node();
        match node.data() {
            &Some(ref data) => {
                let m = data.clone().into();
                match m {
                    Move::Resign | Move::None => {}
                    Move::Play { color, pos } => {
                        board[pos] = color.into();
                        node.incoming()[0].fill_board(board);
                    }
                }
            }
            &None => {}
        }
    }
}
