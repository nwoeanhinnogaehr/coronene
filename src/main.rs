#![feature(slice_patterns)]
#![feature(conservative_impl_trait)]

extern crate rand;
extern crate time;
extern crate bit_vec;
extern crate union_find;
extern crate fnv;

mod player;

use player::htp::HTP;
use player::mctsplayer::MCTSPlayer;
use std::io;

fn main() {
    let player = MCTSPlayer::new();
    let (stdin, stdout) = (io::stdin(), io::stdout());
    let mut htp = HTP::new(stdin.lock(), stdout.lock());
    htp.run(player);
}
