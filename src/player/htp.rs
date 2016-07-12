use std::io::{BufRead, Write};
use super::board::{Move, Pos, Coord};
use super::Player;
use std::fmt::Display;

// like try!, but it sends the error over htp and continues to the next command
macro_rules! try_htp {
    ($htp:ident, $e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                $htp.write_err(e);
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
                    let m = if pos == "resign" {
                        Move::Resign
                    } else {
                        let color = try_htp!(self, color.parse().map_err(|_| "invalid color"));
                        let pos = try_htp!(self, pos.parse::<Pos>().map_err(|_| "invalid move"));
                        Move::new(color, pos)
                    };
                    if player.play_move(m) {
                        self.write_ok("")
                    } else {
                        self.write_err("invalid move")
                    }
                }
                ["undo"] => {
                    player.undo();
                    self.write_ok("")
                }
                ["showboard"] => self.write_ok(player.board()),
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
                    self.write_ok("");
                    break;
                }
                ["final_score"] => {
                    if let Some(color) = player.board().winner() {
                        self.write_ok(color);
                    } else {
                        self.write_err("game is not finished!");
                    }
                }
                _ => self.write_err("syntax error"),
            }
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

    fn write_ok<T>(&mut self, msg: T)
        where T: Display
    {
        write!(self.output, "= {}\n\n", msg).unwrap()
    }
    fn write_err<T>(&mut self, msg: T)
        where T: Display
    {
        write!(self.output, "? {}\n\n", msg).unwrap()
    }
}
