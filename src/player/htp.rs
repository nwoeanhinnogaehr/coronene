use std::io::{BufRead, Write};
use super::board::{Move, Pos, Coord};
use super::Player;
use std::process;
use std::io;

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

    pub fn listen<F>(&mut self, mut fun: F)
        where F: FnMut(String) -> Result<String, String>
    {
        let mut buf = String::new();
        while self.input.read_line(&mut buf).unwrap() > 0 {
            match fun(buf.clone()) {
                Ok(s) => write!(self.output, "= {}\n\n", s).unwrap(),
                Err(s) => write!(self.output, "? {}\n\n", s).unwrap(),
            };
            buf.clear();
        }
    }

    pub fn run<P>(mut self, mut player: P)
        where P: Player
    {
        self.listen(move |cmd_str| {
            let words = cmd_str.split_whitespace();
            let command = match words.collect::<Vec<&str>>()[..] {
                ["genmove", color] => {
                    let color = try!(color.parse().map_err(|_| "invalid color"));
                    Ok(format!("{}", player.generate_move(color).pos))
                }
                ["play", color, pos] => {
                    let color = try!(color.parse().map_err(|_| "invalid color"));
                    let pos = try!(pos.parse::<Pos>().map_err(|_| "invalid move"));
                    if player.play_move(Move::new(color, pos)) {
                        Ok("".into())
                    } else {
                        Err("invalid move")
                    }
                }
                ["showboard"] => Ok(format!("\n{}", player.board())),
                ["name"] => Ok(player.name()),
                ["version"] => Ok(player.version()),
                ["hexgui-analyze_commands"] => Ok("".into()),
                ["boardsize", cols, rows] => {
                    let cols = try!(cols.parse::<Coord>().map_err(|_| "invalid size"));
                    let rows = try!(rows.parse::<Coord>().map_err(|_| "invalid size"));
                    if player.board().cols() == cols && player.board().rows() == rows {
                        Ok("".into())
                    } else {
                        Err("unsupportede boardsize")
                    }
                }
                ["quit"] => {
                    // quick hack fixme
                    let stdout = io::stdout();
                    write!(stdout.lock(), "= \n\n").unwrap();
                    process::exit(0);
                }
                _ => Err("syntax error"),
            };
            command.map_err(|x| x.into())
        });
    }
}
