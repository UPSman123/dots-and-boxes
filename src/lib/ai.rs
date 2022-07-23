use web_sys::console;

use crate::lib::{BarDirection, BarId, BoardState};

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

trait MinMaxState {
    type Move: Copy;

    fn possible_moves(&self) -> Vec<Self::Move>;
}

impl MinMaxState for BoardState {
    type Move = BarId;

    fn possible_moves(&self) -> Vec<Self::Move> {
        vec![]
    }
}

struct MinMaxEngine<T: MinMaxState> {
    cur_state: T,
}

impl<T: MinMaxState> MinMaxEngine<T> {
    fn new(cur_state: T) -> Self {
        Self { cur_state }
    }

    fn best_move(&mut self, state: &T) -> Option<T::Move> {
        state.possible_moves().first().cloned()
    }
}
