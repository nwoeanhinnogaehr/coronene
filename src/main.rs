#![feature(slice_patterns)]

extern crate rand;

mod player;

use player::htp::HTP;
use player::RandomPlayer;
use std::io;

fn main() {
    let player = RandomPlayer::new();
    let (stdin, stdout) = (io::stdin(), io::stdout());
    let mut htp = HTP::new(stdin.lock(), stdout.lock());
    htp.run(player);
}
