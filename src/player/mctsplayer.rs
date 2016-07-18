use super::Player;
use super::board::{Board, Color, Move, Coord, Pos};
use super::graph::{NodeRef, Node};
use std::f32;
use time;
use rand::{self, thread_rng, ThreadRng, Rng};
use std::thread;

const EXPLORATION: f32 = f32::consts::SQRT_2;
const SEARCH_TIME: f32 = 1.0;
const NUM_THREADS: usize = 2;

#[derive(Clone, Debug)]
struct MCTSNode {
    m: Move,
    n: isize,
    q: isize,
}

impl MCTSNode {
    pub fn new(m: Move) -> MCTSNode {
        MCTSNode { m: m, n: 0, q: 0 }
    }
    pub fn get_move(&self) -> Move {
        self.m
    }
    pub fn win_rate(&self) -> f32 {
        self.q as f32 / self.n as f32
    }
}

impl Node<MCTSNode> {
    pub fn value(&self) -> f32 {
        let data = self.data();
        if data.n == 0 {
            if EXPLORATION == 0.0 {
                0.0
            } else {
                f32::INFINITY
            }
        } else {
            let parent = self.parent().unwrap().upgrade();
            let parent_n = parent.get().data().n as f32;
            let q = data.q as f32;
            let n = data.n as f32;
            q / n + EXPLORATION * (2.0 * parent_n.ln() / n).sqrt()
        }
    }
}

struct SearchThread {
    board: Board,
    tree: NodeRef<MCTSNode>,
}

impl SearchThread {
    fn new(board: Board, tree: NodeRef<MCTSNode>) -> SearchThread {
        SearchThread {
            board: board,
            tree: tree,
        }
    }
    fn search(&mut self, max_time: f32) {
        let start_time = time::precise_time_s();
        let mut num_rollouts = 0;
        while time::precise_time_s() - start_time < max_time as f64 {
            let (node, mut state) = self.select_node();
            let outcome = self.roll_out(&mut state);
            self.back_up(node, outcome);
            num_rollouts += 1;
        }
        eprintln!("Tree size: {}", self.tree.get().tree_size());
        eprintln!("Num rollouts: {}", num_rollouts);
    }

    fn select_node(&mut self) -> (NodeRef<MCTSNode>, Board) {
        let mut node = self.tree.clone();
        let mut state = self.board.clone();
        while node.get().children().len() != 0 {
            // find the child with the max value
            let mut max_node = node.get().children()[0].clone();
            let mut max_value = f32::NEG_INFINITY;
            for child in node.get().children() {
                let child_value = child.get().value();
                if child_value > max_value {
                    max_node = child.clone();
                    max_value = child_value;
                }
            }
            node = max_node;
            state.play(node.get().data().get_move());
            // if it hasn't been visited, it's the one
            if node.get().data().n == 0 {
                return (node, state);
            }
        }
        if state.winner().is_none() {
            self.expand(state.to_play(), &node, &state);
            let new_node = thread_rng().choose(node.get().children()).cloned().unwrap();
            node = new_node;
            state.play(node.get().data().get_move());
        }
        (node, state)
    }

    fn roll_out(&mut self, state: &mut Board) -> Color {
        let mut empty_cells: Vec<Pos> = state.empty_cells().collect();
        loop {
            let must_play = self.must_play(state);
            let m = match must_play {
                Move::Resign => break,
                Move::None => {
                    let pos_idx = thread_rng().gen_range(0, empty_cells.len());
                    let pos = empty_cells.remove(pos_idx);
                    Move::new(state.to_play(), pos)
                }
                Move::Play { pos, color: _ } => {
                    let idx = empty_cells.iter().position(|&x| x == pos).unwrap();
                    empty_cells.remove(idx);
                    must_play
                }
            };
            if !state.play(m) {
                panic!("roll out chose filled cell!");
            }
        }
        state.winner().unwrap()
    }

    fn must_play(&mut self, state: &Board) -> Move {
        // game over, must play resign
        if state.winner().is_some() {
            return Move::Resign;
        }

        // save bridge
        let last_move = state.last_move();
        if let Move::Play { pos, color } = last_move {
            let neighbor_patterns = &[(-1, 0), (0, -1), (1, -1), (1, 0), (0, 1), (-1, 1)];
            let num_pat = neighbor_patterns.len();
            let start = thread_rng().gen_range(0, num_pat);
            for i in start..(num_pat + start) {
                let (end_a, end_b) = (pos + neighbor_patterns[i % num_pat].into(),
                                      pos + neighbor_patterns[(i + 2) % num_pat].into());
                let resp = pos + neighbor_patterns[(i + 1) % num_pat].into();
                if state.get(end_a) == Some(state.to_play()) &&
                   state.get(end_a) == state.get(end_b) &&
                   state.get(resp).is_none() {
                    return Move::new(state.to_play(), resp);
                }
            }
        }

        // no mustplay
        Move::None
    }

    fn expand(&mut self, color: Color, node: &NodeRef<MCTSNode>, state: &Board) {
        node.add_children(state.empty_cells()
                               .map(|pos| NodeRef::new(MCTSNode::new(Move::new(color, pos))))
                               .collect())
    }

    fn back_up(&mut self, mut node: NodeRef<MCTSNode>, outcome: Color) {
        let mut reward = if outcome == node.get().data().get_move().color().unwrap() {
            1
        } else {
            0
        };
        loop {
            {
                let mut node_lock = node.get_mut();
                let mut data = node_lock.data_mut();
                data.n += 1;
                data.q += reward;
            }
            reward = 1 - reward;
            let has_parent = node.get().parent().is_some();
            if has_parent {
                let new_node = node.get().parent().unwrap().upgrade();
                node = new_node;
            } else {
                break;
            }
        }
    }
}

pub struct MCTSPlayer {
    board: Board,
    tree: NodeRef<MCTSNode>,
    moves: Vec<Move>,
    rng: ThreadRng,
}

impl MCTSPlayer {
    pub fn new() -> MCTSPlayer {
        MCTSPlayer {
            board: Board::new((13, 13)),
            tree: NodeRef::new(MCTSNode::new(Move::None)),
            moves: Vec::new(),
            rng: rand::thread_rng(),
        }
    }

    /// Return the best move according to the current search tree.
    fn best_move(&mut self) -> Move {
        // choose the node with the largest number of visits
        let node = self.tree.get();
        let max = node.children().iter().map(|x| x.get().data().n).max().unwrap();
        let max_nodes = node.children()
                            .iter()
                            .filter(|x| x.get().data().n == max);
        let best_node = rand::sample(&mut self.rng, max_nodes, 1)[0];
        eprintln!("Win rate {}", best_node.get().data().win_rate());
        return best_node.get().data().get_move();
    }

    fn search(&mut self, max_time: f32) {
        let mut threads = Vec::new();
        for _ in 0..NUM_THREADS {
            let board = self.board.clone();
            let tree = self.tree.clone();
            threads.push(thread::spawn(move || {
                let mut st = SearchThread::new(board, tree);
                st.search(max_time);
            }));
        }
        for t in threads {
            t.join().unwrap();
        }
    }

    fn clear_tree(&mut self) {
        self.tree = NodeRef::new(MCTSNode::new(Move::None));
    }
}

impl Player for MCTSPlayer {
    fn generate_move(&mut self, color: Color) -> Move {
        if self.board.winner().is_some() {
            return Move::Resign;
        }

        if color != self.board.to_play() {
            self.board.set_to_play(color);
            self.clear_tree();
        }
        self.search(SEARCH_TIME);
        let m = self.best_move();
        self.play_move(m);
        m
    }

    /// Force a move.
    fn play_move(&mut self, m: Move) -> bool {
        let node = self.tree
                       .get()
                       .children()
                       .iter()
                       .find(|x| x.get().data().get_move() == m)
                       .cloned();
        if let Some(new_root) = node {
            // if the move is in the tree, make it the new root
            new_root.get_mut().orphan();
            self.tree = new_root;
        } else {
            // otherwise, make a new root
            self.clear_tree();
        }
        self.moves.push(m);
        self.board.play(m)
    }

    fn undo(&mut self) {
        if let Some(Move::Play { pos, color: _}) = self.moves.pop() {
            self.board.clear_cell(pos);
            self.clear_tree();
        }
    }

    fn board(&self) -> &Board {
        &self.board
    }

    fn name(&self) -> String {
        "coronene mcts".into()
    }

    fn version(&self) -> String {
        "0.1".into()
    }

    fn set_board_size(&mut self, cols: Coord, rows: Coord) {
        self.board = Board::new((cols, rows));
        self.clear_tree();
        self.moves.clear();
    }
}
