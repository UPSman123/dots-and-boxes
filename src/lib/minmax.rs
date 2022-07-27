use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use web_sys::console;
use std::fmt::Debug;

pub trait PossibleMovesIterator<S: MinMaxState, M> {
    fn new<'a>(state: &'a S) -> Self;
    fn next<'a>(&mut self, state: &'a S) -> Option<M>;
}

pub struct PossibleMovesWrapper<'a, S: MinMaxState, M, I: PossibleMovesIterator<S, M>> {
    state: &'a S,
    iter: I,
    phantom: PhantomData<M>,
}

impl<'a, S: MinMaxState, M, I: PossibleMovesIterator<S, M>> Iterator
    for PossibleMovesWrapper<'a, S, M, I>
{
    type Item = M;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next(self.state)
    }
}

pub trait MinMaxState: Sized {
    type Move: Copy + Debug;
    type PossibleMovesIterator: PossibleMovesIterator<Self, Self::Move>;

    fn possible_moves<'a>(
        &'a self,
    ) -> PossibleMovesWrapper<'a, Self, Self::Move, Self::PossibleMovesIterator> {
        PossibleMovesWrapper {
            state: self,
            iter: Self::PossibleMovesIterator::new(self),
            phantom: PhantomData::default(),
        }
    }

    fn checkpoint(&mut self) -> MinMaxStateCheckpoint<'_, Self> {
        MinMaxStateCheckpoint {
            state: self,
            mutation_count: 0,
        }
    }

    fn _apply_move(&mut self, mv: Self::Move) -> bool;

    fn _undo_moves(&mut self, nr_moves: u32) -> bool;
}

pub struct MinMaxStateCheckpoint<'a, S: MinMaxState> {
    state: &'a mut S,
    mutation_count: u32,
}

impl<S: MinMaxState> Deref for MinMaxStateCheckpoint<'_, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl<S: MinMaxState> DerefMut for MinMaxStateCheckpoint<'_, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

impl<'a, S: MinMaxState> Drop for MinMaxStateCheckpoint<'a, S> {
    fn drop(&mut self) {
        assert!(self.state._undo_moves(self.mutation_count), "undo move failed");
    }
}

impl<'a, S: MinMaxState> MinMaxStateCheckpoint<'a, S> {
    pub fn apply(&mut self, mv: S::Move) -> bool {
        self.mutation_count += 1;
        assert!(self.state._apply_move(mv), "applying move failed");
        true
    }
}

pub trait MinMaxInterface {
    type State: MinMaxState;

    fn heuristic(&mut self, state: &mut Self::State) -> i32;
}

pub struct MinMaxOptions {}

impl Default for MinMaxOptions {
    fn default() -> Self {
        Self {}
    }
}

pub struct MinMax<I: MinMaxInterface> {
    game: I,
    root_state: I::State,
    options: MinMaxOptions,
}

impl<I: MinMaxInterface> MinMax<I> {
    pub fn new(game: I, root_state: I::State) -> Self {
        Self {
            game,
            root_state,
            options: Default::default(),
        }
    }

    pub fn set_root_state(&mut self, new_root_state: I::State) {
        self.root_state = new_root_state;
    }

    pub fn set_options(&mut self, options: MinMaxOptions) {
        self.options = options;
    }

    pub fn best_move(&mut self) -> Option<<I::State as MinMaxState>::Move> {
        let possible_moves = self.root_state.possible_moves().collect::<Vec<_>>();
        possible_moves.into_iter().map(|mv| {
            let mut state = self.root_state.checkpoint();
            state.apply(mv);
            let real_state: &mut I::State = &mut state;
            let heuristic = self.game.heuristic(real_state);
            (mv, heuristic)
        }).max_by_key(|tup| {
            let (_mv, heuristic) = *tup;
            heuristic
        }).map(|tup| {
            let (mv, _heuristic) = tup;
            mv
        })
    }
}
