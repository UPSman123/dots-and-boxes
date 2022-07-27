use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use std::ops::Deref;
use web_sys::console;

use crate::lib::minmax::*;
use crate::lib::{BarDirection, BarId, BarVec, BoardState, CellState, Player};

pub type AIOptions = MinMaxOptions;

pub trait AI {
    fn new(board_state: &BoardState, ai_player: Player) -> Self;
    fn set_options(&mut self, options: AIOptions);
    fn next_move(&mut self, board_state: &BoardState) -> Option<BarId>;
}

mod intern {
    use super::*;

    pub struct PossibleMovesIter {
        cur_index: u32,
    }

    impl PossibleMovesIterator<AIState, BarId> for PossibleMovesIter {
        fn new<'a>(_state: &'a AIState) -> Self {
            Self { cur_index: 0 }
        }

        fn next<'a>(&mut self, state: &'a AIState) -> Option<BarId> {
            let first_free_from_index = |start_index: u32, vec: &BarVec| {
                (start_index..vec.length).find_map(|index| {
                    let bar_id = vec.index_to_id(index);
                    let cell_state = vec.get(bar_id.col, bar_id.row);
                    if cell_state != CellState::Free {
                        None
                    } else {
                        Some((bar_id, index))
                    }
                })
            };

            let board_state: &BoardState = state;
            let mut cur_index = self.cur_index;

            if cur_index < board_state.vstates.length {
                let first_free_vstate = first_free_from_index(cur_index, &board_state.vstates);
                if let Some((bar_id, index)) = first_free_vstate {
                    self.cur_index = index + 1;
                    return Some(bar_id);
                } else {
                    cur_index = 0;
                }
            } else {
                cur_index -= board_state.vstates.length;
            }
            if cur_index < board_state.hstates.length {
                let first_free_hstate = first_free_from_index(cur_index, &board_state.hstates);
                if let Some((bar_id, index)) = first_free_hstate {
                    self.cur_index = index + board_state.vstates.length + 1;
                    return Some(bar_id);
                }
            }
            None
        }
    }

    pub struct AIState {
        board_state: BoardState,
        mutation_stack: Vec<BarId>,
    }

    impl MinMaxState for AIState {
        type Move = BarId;
        type PossibleMovesIterator = PossibleMovesIter;

        fn _apply_move(&mut self, mv: Self::Move) -> bool {
            self.mutation_stack.push(mv);
            self.board_state.do_move(mv)
        }

        fn _undo_moves(&mut self, nr_moves: u32) -> bool {
            (0..nr_moves).all(|_| {
                let mv = self
                    .mutation_stack
                    .pop()
                    .expect("mutation stack empty during undo");
                let neighbors = self.board_state.bar_neighbors(mv);
                let point_scored = neighbors.iter().any(|tup| {
                    let (col, row) = *tup;
                    let cell_state = self.board_state.cell_get(col, row);
                    assert!(
                        cell_state != CellState::Player(self.board_state.cur_turn.other()),
                        "wrong cell state, cell_state: {:?}, cur_turn: {:?}",
                        cell_state,
                        self.board_state.cur_turn,
                    );
                    cell_state != CellState::Free
                });
                let this_turn = if point_scored {
                    self.board_state.cur_turn
                } else {
                    self.board_state.cur_turn.other()
                };
                let vec = match mv.direction {
                    BarDirection::Vertical => &mut self.board_state.vstates,
                    BarDirection::Horizontal => &mut self.board_state.hstates,
                };
                let state = vec.get(mv.col, mv.row);
                if state != CellState::Player(this_turn) {
                    // panic!(
                    //     "state: {:?}, cur_player: {:?}, this turn: {:?}",
                    //     state, self.board_state.cur_turn, this_turn
                    // );
                    return false;
                }

                vec.set(mv.col, mv.row, CellState::Free);
                for cell in neighbors.into_iter() {
                    self.board_state.cell_set(cell.0, cell.1, CellState::Free);
                }
                self.board_state.cur_turn = this_turn;
                true
            })
        }
    }

    impl From<BoardState> for AIState {
        fn from(board_state: BoardState) -> AIState {
            Self {
                board_state,
                mutation_stack: vec![],
            }
        }
    }

    impl Deref for AIState {
        type Target = BoardState;

        fn deref(&self) -> &Self::Target {
            &self.board_state
        }
    }

    pub fn random_free_move<R: Rng>(state: &AIState, rng: &mut R) -> Option<BarId> {
        let total_bars = state.vstates.length + state.hstates.length;
        for _ in 0..3 {
            let chosen_index = rng.gen_range(0..total_bars);
            let (vec, index) = if chosen_index < state.vstates.length {
                (&state.vstates, chosen_index)
            } else {
                (&state.hstates, chosen_index - state.vstates.length)
            };
            let bar_id = vec.index_to_id(index);
            let state = vec.get(bar_id.col, bar_id.row);
            if state == CellState::Free {
                return Some(bar_id);
            }
        }
        let possible_moves = state.possible_moves().collect::<Vec<_>>();
        possible_moves.choose(rng).cloned()
    }
}

pub struct AIMinMaxInterface {
    options: AIOptions,
    ai_player: Player,
    rng: ThreadRng,
}

impl AIMinMaxInterface {
    fn new(ai_player: Player) -> Self {
        let rng = rand::thread_rng();
        let options = Default::default();
        Self {
            options,
            ai_player,
            rng,
        }
    }
}

impl MinMaxInterface for AIMinMaxInterface {
    type State = intern::AIState;

    fn heuristic(&mut self, state: &mut Self::State) -> i32 {
        let nr_tests = 5;
        let results = (0..nr_tests).map(|_| {
            let mut cur_state = state.checkpoint();
            while let Some(mv) = intern::random_free_move(&cur_state, &mut self.rng) {
                cur_state.apply(mv);
            }
            let score: i32 = cur_state
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
            score
        });
        results.sum::<i32>()
    }
}

pub type AIMinMax = MinMax<AIMinMaxInterface>;

impl AI for AIMinMax {
    fn new(board_state: &BoardState, ai_player: Player) -> Self {
        let game = AIMinMaxInterface::new(ai_player);
        let root_state = board_state.clone().into();
        Self::new(game, root_state)
    }

    fn set_options(&mut self, options: AIOptions) {
        self.set_options(options);
    }

    fn next_move(&mut self, state: &BoardState) -> Option<BarId> {
        self.set_root_state(state.clone().into());
        self.best_move()
    }
}
