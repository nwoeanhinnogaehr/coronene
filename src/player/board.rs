use std::ops::{Index, IndexMut};
use std::fmt;

#[derive(Copy, Clone, Debug)]
pub enum Color {
    Black,
    White,
}

#[derive(Copy, Clone, Debug)]
pub enum Cell {
    Empty,
    Black,
    White,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Cell::Empty => write!(f, "."),
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

    pub fn parse(s: &str) -> Option<Pos> {
        let b = s.as_bytes();
        let x = if b[0] >= b'a' && b[0] <= b'z' {
            b[0] - b'a'
        } else if b[0] >= b'A' && b[0] <= b'Z' {
            b[0] - b'A'
        } else {
            return None;
        };
        let y = match s[1..].parse::<Coord>() {
            Ok(y) => y - 1,
            Err(_) => return None,
        };
        Some(Pos::new(x, y))
    }
}

impl From<Pos> for (Coord, Coord) {
    fn from(p: Pos) -> (Coord, Coord) {
        (p.x, p.y)
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", (self.x + 'A' as Coord) as char, self.y + 1)
    }
}

#[derive(Copy, Clone)]
pub struct Move {
    pub color: Color,
    pub pos: Pos,
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

    fn idx_of(&self, x: Coord, y: Coord) -> Option<usize> {
        if x >= self.cols || y >= self.rows {
            None
        } else {
            Some(y as usize * self.rows as usize + x as usize)
        }
    }
}

impl<T> Index<T> for Board
    where T: Into<(Coord, Coord)>
{
    type Output = Cell;

    fn index(&self, idx: T) -> &Self::Output {
        let idx = idx.into();
        let idx = self.idx_of(idx.0, idx.1).expect("board index out of bounds");
        &self.board[idx]
    }
}

impl<T> IndexMut<T> for Board
    where T: Into<(Coord, Coord)>
{
    fn index_mut(&mut self, idx: T) -> &mut Self::Output {
        let idx = idx.into();
        let idx = self.idx_of(idx.0, idx.1).expect("board index out of bounds");
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
            for x in 0..self.cols-1 {
                try!(write!(f, "{} ", self[(x, y)]));
            }
            try!(write!(f, "{}\\{}", self[(self.cols-1, y)], y + 1));
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
