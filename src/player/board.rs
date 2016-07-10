use std::fmt;
use std::str::FromStr;
use bit_vec::{self, BitVec};
use union_find::{UnionFind, UnionBySize, QuickUnionUf};

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

impl Color {
    pub fn invert(&self) -> Color {
        match self {
            &Color::Black => Color::White,
            &Color::White => Color::Black,
        }
    }
}

impl From<bool> for Color {
    fn from(v: bool) -> Color {
        if v {
            Color::White
        } else {
            Color::Black
        }
    }
}

impl From<Color> for bool {
    fn from(v: Color) -> bool {
        v == Color::White
    }
}

pub type Coord = u8;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Pos {
    x: Coord,
    y: Coord,
}

impl Pos {
    pub fn new(x: Coord, y: Coord) -> Pos {
        Pos { x: x, y: y }
    }

    pub fn area(&self) -> usize {
        self.x as usize * self.y as usize
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

#[derive(Copy, Clone, PartialEq, Debug)]
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

#[derive(Clone, Debug)]
pub struct Board {
    dims: Pos,
    colors: BitVec,
    empty_cells: BitVec,
    to_play: Color,
    groups: Vec<QuickUnionUf<UnionBySize>>,
}

impl Board {
    pub fn new<P: Into<Pos>>(dims: P) -> Board {
        let dims = dims.into();
        Board {
            dims: dims,
            colors: BitVec::from_elem(dims.area(), false),
            empty_cells: BitVec::from_elem(dims.area(), true),
            to_play: Color::Black,
            groups: vec![QuickUnionUf::new(dims.area() + 2); 2],
        }
    }

    pub fn dimensions(&self) -> Pos {
        self.dims
    }

    pub fn is_empty<P>(&self, pos: P) -> bool
        where P: Into<Pos>
    {
        self.get(pos) == None
    }

    pub fn clear_cell<P>(&mut self, pos: P)
        where P: Into<Pos>
    {
        self.set(pos, None);
    }

    pub fn play(&mut self, m: Move) -> bool {
        match m {
            Move::Resign | Move::None => true,
            Move::Play { color, pos } => {
                if !self.is_empty(pos) {
                    false
                } else {
                    self.set(pos, Some(color));
                    self.to_play = color.invert();
                    true
                }
            }
        }
    }

    pub fn empty_cells<'a>(&'a self) -> Iter<'a> {
        Iter {
            iter: self.empty_cells.iter(),
            idx: 0,
            dims: self.dims,
        }
    }

    pub fn check_win(&mut self) -> Option<Color> {
        let edge0 = self.edge_idx(0);
        let edge1 = self.edge_idx(1);
        for i in 0..2 {
            if self.groups[i].find(edge0) == self.groups[i].find(edge1) {
                return Some((i > 0).into());
            }
        }
        None
    }

    pub fn on_board<P>(&self, pos: P) -> bool
        where P: Into<Pos>
    {
        let pos = pos.into();
        pos.x < self.dims.x && pos.y < self.dims.y
    }

    pub fn get<P: Into<Pos>>(&self, pos: P) -> Option<Color> {
        let pos = pos.into();
        let idx = self.idx_of(pos).expect("board index out of bounds");
        if self.empty_cells[idx] {
            None
        } else {
            Some(self.colors[idx].into())
        }
    }

    fn update_groups<P: Into<Pos>>(&mut self, pos: P) {
        let pos = pos.into();
        let val = match self.get(pos) {
            Some(val) => val,
            None => return,
        };
        let idx = self.idx_of(pos).unwrap();
        let i = val as usize;
        if let Some(edge_idx) = match (pos.x, pos.y, val) {
            (_, 0, Color::Black) | (0, _, Color::White) => Some(self.edge_idx(0)),
            (_, y, Color::Black) if y == self.dims.y - 1 => Some(self.edge_idx(1)),
            (x, _, Color::White) if x == self.dims.x - 1 => Some(self.edge_idx(1)),
            _ => None,
        } {
            self.groups[i].union(idx, edge_idx);
        }
        let neighbor_patterns = &[(-1, 0), (0, -1), (-1, 1), (0, 1), (1, 0), (1, -1)];
        for pat in neighbor_patterns {
            let cell = Pos {
                x: (pos.x as isize + pat.0) as Coord,
                y: (pos.y as isize + pat.1) as Coord,
            };
            if self.on_board(cell) && self.get(cell) == Some(val) {
                let connection_idx = self.idx_of(cell).unwrap();
                self.groups[i].union(idx, connection_idx);
            }
        }
    }

    fn rebuild_groups(&mut self) {
        self.groups = vec![QuickUnionUf::new(self.dims.area() + 2); 2];
        for x in 0..self.dims.x {
            for y in 0..self.dims.y {
                self.update_groups((x, y));
            }
        }
    }

    pub fn set<P: Into<Pos>>(&mut self, pos: P, val: Option<Color>) {
        let pos = pos.into();
        let idx = self.idx_of(pos).expect("board index out of bounds");
        self.empty_cells.set(idx, val.is_none());
        if let Some(color) = val {
            self.colors.set(idx, color.into());
            self.update_groups(pos);
        } else {
            self.rebuild_groups();
        }
    }

    pub fn to_play(&self) -> Color {
        self.to_play
    }

    pub fn set_to_play(&mut self, color: Color) {
        self.to_play = color;
    }

    fn idx_of<P: Into<Pos>>(&self, pos: P) -> Option<usize> {
        let pos = pos.into();
        if self.on_board(pos) {
            Some(pos.y as usize * self.dims.y as usize + pos.x as usize)
        } else {
            None
        }
    }

    fn edge_idx(&self, edge: usize) -> usize {
        self.dims.area() + edge
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "  "));
        for x in 0..self.dims.x {
            try!(write!(f, "{} ", (x + 'a' as Coord) as char));
        }
        for y in 0..self.dims.y {
            try!(write!(f, "\n"));
            for _ in 0..y {
                try!(write!(f, " "));
            }
            try!(write!(f, "{:2}\\", y + 1));
            for x in 0..self.dims.x {
                match self.get((x, y)) {
                    Some(c) => try!(write!(f, "{}", c)),
                    None => try!(write!(f, "+")),
                }
                if x != self.dims.x - 1 {
                    try!(write!(f, " "));
                }
            }
            try!(write!(f, "\\{}", y + 1));
        }
        try!(write!(f, "\n   "));
        for _ in 0..self.dims.y {
            try!(write!(f, " "));
        }
        for x in 0..self.dims.x {
            try!(write!(f, "{} ", (x + 'a' as Coord) as char));
        }
        Ok(())
    }
}

pub struct Iter<'a> {
    iter: bit_vec::Iter<'a>,
    idx: usize,
    dims: Pos,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Pos;

    fn next(&mut self) -> Option<Pos> {
        while let Some(val) = self.iter.next() {
            if val {
                let res = Some(((self.idx % self.dims.x as usize) as Coord,
                                (self.idx / self.dims.y as usize) as Coord)
                                   .into());
                self.idx += 1;
                return res;
            }
            self.idx += 1;
        }
        None
    }
}
