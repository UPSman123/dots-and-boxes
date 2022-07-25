use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use web_sys::console;

use crate::lib::{BarId, BarVecIdIterator, BoardState, CellState, Player};

pub struct AIOptions {}

impl Default for AIOptions {
    fn default() -> Self {
        Self {}
    }
}

pub trait AI {
    fn new(board_state: &BoardState, ai_player: Player) -> Self;

    fn options_set(&mut self, options: AIOptions);

    fn next_move(&mut self, board_state: &BoardState) -> Option<BarId>;
}

pub struct SimpleAI {
    options: AIOptions,
    cur_state: BoardState,
    ai_player: Player,
    rng: ThreadRng,
}

impl AI for SimpleAI {
    fn new(board_state: &BoardState, ai_player: Player) -> Self {
        let rng = rand::thread_rng();
        let options = Default::default();
        Self {
            options,
            cur_state: board_state.clone(),
            ai_player,
            rng,
        }
    }

    fn options_set(&mut self, options: AIOptions) {
        self.options = options;
    }

    fn next_move(&mut self, state: &BoardState) -> Option<BarId> {
        console::log_1(&"calculating AI move".into());
        self.cur_state = state.clone();
        let best_move = state.possible_moves().max_by_key(|mv: &BarId| {
            let mv = mv.clone();
            assert!(self.cur_state.apply_move(mv), "Couldn't apply AI move");
            let score = self.heuristic();
            console::log_1(&format!("move: {:?}, score: {:?}", mv, score).into());
            assert!(self.cur_state.undo_move(mv), "Couldn't undo AI move");
            score
        });
        best_move
    }
}

impl SimpleAI {
    fn heuristic(&mut self) -> i32 {
        let nr_tests = 20;
        let results = (0..nr_tests).map(|_| {
            let mut move_stack = vec![];
            while let Some(mv) = self.cur_state.random_free_move(&mut self.rng) {
                self.cur_state.apply_move(mv);
                move_stack.push(mv);
            }
            let score: i32 = self
                .cur_state
                .cellstates
                .iter()
                .map(|cell_state| match cell_state.clone() {
                    CellState::Free => panic!("found free cell in completed board"),
                    CellState::Player(player) => {
                        if player == self.ai_player {
                            1
                        } else {
                            -1
                        }
                    }
                })
                .sum();
            while let Some(mv) = move_stack.pop() {
                self.cur_state.undo_move(mv);
            }
            score
        });
        results.sum::<i32>()
    }
}

struct PossibleMovesIter<'a> {
    iterator: InternalMovesIter<'a>,
}

type InternalMovesIter<'a> = std::iter::Chain<BarVecIdIterator<'a>, BarVecIdIterator<'a>>;

impl<'a> PossibleMovesIter<'a> {
    fn new(board: &'a BoardState) -> Self {
        let vertical_bars = board.vstates.iter();
        let horizontal_bars = board.hstates.iter();
        let combined = vertical_bars.chain(horizontal_bars);

        Self { iterator: combined }
    }
}

impl<'a> Iterator for PossibleMovesIter<'a> {
    type Item = BarId;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.find_map(|tup: (BarId, CellState)| {
            if tup.1 == CellState::Free {
                Some(tup.0)
            } else {
                None
            }
        })
    }
}

trait AIBoardState: Clone {
    fn possible_moves(&self) -> PossibleMovesIter<'_>;
    fn apply_move(&mut self, mv: BarId) -> bool;
    fn undo_move(&mut self, mv: BarId) -> bool;
    fn random_free_move<R: Rng>(&self, rng: &mut R) -> Option<BarId>;
}

impl AIBoardState for BoardState {
    fn possible_moves(&self) -> PossibleMovesIter<'_> {
        PossibleMovesIter::new(self)
    }

    fn apply_move(&mut self, mv: BarId) -> bool {
        self.do_move(mv)
    }

    fn undo_move(&mut self, mv: BarId) -> bool {
        let cur_state = self.bar_get(mv);
        if cur_state == CellState::Free {
            false
        } else {
            self.bar_set(mv, CellState::Free);
            true
        }
    }

    fn random_free_move<R: Rng>(&self, rng: &mut R) -> Option<BarId> {
        let total_bars = self.vstates.length + self.hstates.length;
        for _ in 0..3 {
            let chosen_index = rng.gen_range(0..total_bars);
            let (vec, index) = if chosen_index < self.vstates.length {
                (&self.vstates, chosen_index)
            } else {
                (&self.hstates, chosen_index - self.vstates.length)
            };
            let bar_id = vec.index_to_id(index);
            let state = vec.get(bar_id.col, bar_id.row);
            if state == CellState::Free {
                return Some(bar_id);
            }
        }
        let possible_moves = self.possible_moves().collect::<Vec<_>>();
        possible_moves.choose(rng).cloned()
    }
}
