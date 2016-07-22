use super::Player;
use super::board::{Board, Color, Move, Coord, Pos};
use super::graph::{NodeRef, Node};
use std::f32;
use time;
use rand::{self, thread_rng, Rng};
use std::thread;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::collections::HashSet;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

const EXPLORATION: f32 = f32::consts::SQRT_2;
const SEARCH_TIME: f32 = 1.0;
const NUM_THREADS: usize = 1;

#[derive(Debug)]
struct Stats {
    n: AtomicIsize,
    q: AtomicIsize,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            n: AtomicIsize::new(0),
            q: AtomicIsize::new(0),
        }
    }

    pub fn mean(&self) -> f32 {
        let n = self.n();
        if n == 0 {
            0.5 // TODO what should this be??
        } else {
            self.q() as f32 / n as f32
        }
    }

    pub fn n(&self) -> isize {
        self.n.load(Ordering::SeqCst)
    }

    pub fn q(&self) -> isize {
        self.q.load(Ordering::SeqCst)
    }

    pub fn visit(&self, num: isize) {
        self.n.fetch_add(num, Ordering::SeqCst);
    }

    pub fn reward(&self, reward: isize) {
        self.q.fetch_add(reward, Ordering::SeqCst);
    }
}

#[derive(Debug)]
struct MCTSNode {
    action: Move,
    mc: Stats,
    rave: Stats,
}

impl MCTSNode {
    pub fn new(action: Move) -> MCTSNode {
        MCTSNode {
            action: action,
            mc: Stats::new(),
            rave: Stats::new(),
        }
    }
}

impl Node<MCTSNode> {
    pub fn value(&self) -> f32 {
        let data = self.data();
        if data.mc.n() == 0 {
            if EXPLORATION == 0.0 {
                0.0
            } else {
                f32::INFINITY
            }
        } else {
            let parent = self.parent().unwrap().upgrade();
            let parent_n = parent.get().data().mc.n() as f32;
            let n = data.mc.n() as f32;
            let b = 0.5; // TODO tune this
            let rave_n = data.rave.n() as f32;
            let mc_n = data.mc.n() as f32;
            let beta = rave_n / (mc_n + rave_n + 4.0 * mc_n * rave_n * b * b);
            let q = (1.0 - beta) * data.mc.mean() + beta * data.rave.mean();
            q * 2.0 - 1.0 + EXPLORATION * (2.0 * parent_n.ln() / n).sqrt()
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

    /// Run Monte-Carlo search for the given number of seconds.
    fn search(&mut self, max_time: f32) {
        let start_time = time::precise_time_s();
        let mut num_rollouts = 0;
        while time::precise_time_s() - start_time < max_time as f64 {
            let (node, mut state) = self.select_node();
            let outcome = self.roll_out(&mut state);
            self.back_up(node, outcome, &state);
            num_rollouts += 1;
        }
        eprintln!("Num rollouts: {}", num_rollouts);
    }

    /// Monte-Carlo selection process
    fn select_node(&mut self) -> (NodeRef<MCTSNode>, Board) {
        let mut node = self.tree.clone();
        let mut state = self.board.clone();

        // virtual losses: visit node in selection process
        node.get().data().mc.visit(1);

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

            node.get().data().mc.visit(1); // virtual losses
            state.play(node.get().data().action); // simulate the action associated with the move

            // if it hasn't been visited yet, select it
            if node.get().data().mc.n() == 1 {
                return (node, state);
            }
        }

        // if this is not a leaf node (no winner), expand the tree.
        if state.winner().is_none() {
            self.expand(state.to_play(), &node, &state);
            // choose a child randomly
            let new_node = thread_rng().choose(node.get().children()).cloned().unwrap();
            node = new_node;

            node.get().data().mc.visit(1); // virtual losses
            state.play(node.get().data().action); // simulate action
        }

        (node, state)
    }

    /// Simulate a random game from a state and return the winner.
    fn roll_out(&mut self, state: &mut Board) -> Color {
        let mut empty_cells: Vec<Pos> = state.iter_empty().collect();
        loop {
            // check for a must play
            let must_play = self.must_play(state);
            let m = match must_play {
                Move::Resign => break,
                Move::None => {
                    // no must play, pick random move
                    let pos_idx = thread_rng().gen_range(0, empty_cells.len());
                    let pos = empty_cells.remove(pos_idx);
                    Move::new(state.to_play(), pos)
                }
                Move::Play { pos, color: _ } => {
                    // must play, play it
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
        if let Move::Play { pos, color: _ } = last_move {
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

    /// Adds all children (possible moves) to a node.
    fn expand(&mut self, color: Color, node: &NodeRef<MCTSNode>, state: &Board) {
        node.add_children(state.iter_empty()
                               .map(|pos| NodeRef::new(MCTSNode::new(Move::new(color, pos))))
                               .collect())
    }

    /// Propagate roll out results back up the tree
    fn back_up(&mut self, mut node: NodeRef<MCTSNode>, outcome: Color, endgame: &Board) {

        // RAVE needs to keep track of all visited actions
        let hasher = BuildHasherDefault::<FnvHasher>::default();
        let mut actions = HashSet::with_hasher(hasher);
        actions.extend(endgame.iter_filled());

        let mut reward = if Some(outcome) == node.get().data().action.color() {
            1
        } else {
            0
        };
        loop {
            node.get().data().mc.reward(reward);
            actions.insert(node.get().data().action);

            let has_parent = node.get().parent().is_some();
            if has_parent {
                // move up the tree
                let new_node = node.get().parent().unwrap().upgrade();
                node = new_node;
            } else {
                break;
            }

            // RAVE
            for child in node.get().children() {
                if actions.contains(&child.get().data().action) {
                    child.get().data().rave.visit(1);
                    child.get().data().rave.reward(reward);
                }
            }

            reward = 1 - reward; // flip reward for other player
        }
    }
}

pub struct MCTSPlayer {
    board: Board,
    tree: NodeRef<MCTSNode>,
    moves: Vec<Move>,
}

impl MCTSPlayer {
    pub fn new() -> MCTSPlayer {
        MCTSPlayer {
            board: Board::new((13, 13)),
            tree: NodeRef::new(MCTSNode::new(Move::None)),
            moves: Vec::new(),
        }
    }

    /// Return the best move according to the current search tree.
    fn best_move(&mut self) -> Move {
        // choose the node with the largest number of visits
        let node = self.tree.get();
        let max = node.children().iter().map(|x| x.get().data().mc.n()).max().unwrap();
        let max_nodes = node.children()
                            .iter()
                            .filter(|x| x.get().data().mc.n() == max);
        // pick a random move that has the max visits
        let best_node = rand::sample(&mut thread_rng(), max_nodes, 1)[0];
        eprintln!("Win rate {}", best_node.get().data().mc.mean());
        eprintln!("RAVE win rate {}", best_node.get().data().rave.mean());
        return best_node.get().data().action;
    }

    fn search(&mut self, max_time: f32) {
        // spawn search threads
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
        eprintln!("Tree size: {}", self.tree.get().tree_size());
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
                       .find(|x| x.get().data().action == m)
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
