use std::io::{BufRead, Write};
use super::board::{Move, Pos, Coord};
use super::Player;
use std::io;
use std::fmt::Display;

macro_rules! try_htp {
    ($htp:ident, $e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                $htp.write_err(e).unwrap();
                continue;
            }
        }
    };
}

pub struct HTP<R, W>
    where R: BufRead,
          W: Write
{
    input: R,
    output: W,
}

impl<R, W> HTP<R, W>
    where R: BufRead,
          W: Write
{
    pub fn new(input: R, output: W) -> HTP<R, W> {
        HTP {
            input: input,
            output: output,
        }
    }

    pub fn run<P>(&mut self, mut player: P)
        where P: Player
    {
        loop {
            let cmd_str = match self.read() {
                Some(x) => x,
                None => break,
            };
            let words = cmd_str.split_whitespace();
            match words.collect::<Vec<&str>>()[..] {
                ["genmove", color] => {
                    let color = try_htp!(self, color.parse().map_err(|_| "invalid color"));
                    self.write_ok(format!("{}", player.generate_move(color)))
                }
                ["play", color, pos] => {
                    let color = try_htp!(self, color.parse().map_err(|_| "invalid color"));
                    let pos = try_htp!(self, pos.parse::<Pos>().map_err(|_| "invalid move"));
                    if player.play_move(Move::new(color, pos)) {
                        self.write_ok("")
                    } else {
                        self.write_err("invalid move")
                    }
                }
                ["showboard"] => self.write_ok(format!("\n{}", player.board())),
                ["name"] => self.write_ok(player.name()),
                ["version"] => self.write_ok(player.version()),
                ["hexgui-analyze_commands"] => self.write_ok(""),
                ["boardsize", cols, rows] => {
                    let cols = try_htp!(self, cols.parse::<Coord>().map_err(|_| "invalid size"));
                    let rows = try_htp!(self, rows.parse::<Coord>().map_err(|_| "invalid size"));
                    player.set_board_size(cols, rows);
                    self.write_ok("")
                }
                ["quit"] => {
                    self.write_ok("").unwrap();
                    break;
                }
                _ => self.write_err("syntax error"),
            }
            .unwrap();
        }
    }

    fn read(&mut self) -> Option<String> {
        let mut buf = String::new();
        let res = self.input.read_line(&mut buf);
        match res {
            Ok(n) if n > 0 => Some(buf),
            _ => None,
        }
    }

    fn write<T, U>(&mut self, msg: Result<T, U>) -> io::Result<()>
        where T: Display,
              U: Display
    {
        match msg {
            Ok(s) => write!(self.output, "= {}\n\n", s),
            Err(s) => write!(self.output, "? {}\n\n", s),
        }
    }
    fn write_ok<T>(&mut self, msg: T) -> io::Result<()>
        where T: Display
    {
        write!(self.output, "= {}\n\n", msg)
    }
    fn write_err<T>(&mut self, msg: T) -> io::Result<()>
        where T: Display
    {
        write!(self.output, "? {}\n\n", msg)
    }
}
