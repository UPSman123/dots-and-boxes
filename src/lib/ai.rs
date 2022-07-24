use web_sys::console;

use crate::lib::{BarId, BarVecIdIterator, BoardState, CellState};

pub struct AIOptions {}

impl Default for AIOptions {
    fn default() -> Self {
        Self {}
    }
}

pub trait AI {
    fn new(board_state: Option<&BoardState>) -> Self;

    fn set_options(&mut self, options: AIOptions);

    fn next_move(&mut self, board_state: &BoardState) -> Option<BarId>;
}

pub struct SimpleAI {
    min_max_engine: MinMaxEngine<BoardState>,
}

impl AI for SimpleAI {
    fn new(board_state: Option<&BoardState>) -> Self {
        Self {
            min_max_engine: MinMaxEngine::new(board_state.unwrap().clone()),
        }
    }

    fn set_options(&mut self, _options: AIOptions) {}

    fn next_move(&mut self, board_state: &BoardState) -> Option<BarId> {
        console::log_1(&"calculating AI move".into());
        self.min_max_engine.best_move(board_state)
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

trait MinMaxState {
    fn possible_moves(&self) -> PossibleMovesIter<'_>;
}

impl MinMaxState for BoardState {
    fn possible_moves(&self) -> PossibleMovesIter<'_> {
        PossibleMovesIter::new(self)
    }
}

struct MinMaxEngine<T: MinMaxState> {
    cur_state: T,
}

impl<T: MinMaxState> MinMaxEngine<T> {
    fn new(cur_state: T) -> Self {
        Self { cur_state }
    }

    fn best_move(&mut self, state: &T) -> Option<BarId> {
        let moves = state.possible_moves().collect::<Vec<_>>();
        console::log_1(&format!("moves: {:?}", moves).into());
        moves.first().cloned()
    }
}
