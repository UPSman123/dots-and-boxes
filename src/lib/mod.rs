use web_sys::console;
use std::fmt;
use yew::html::IntoPropValue;

mod ai;
mod frontend;

pub use frontend::AppComp;

#[derive(Clone, Copy, Debug)]
pub enum BarDirection {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug)]
pub struct BarId {
    pub direction: BarDirection,
    pub col: u32,
    pub row: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellState {
    Free,
    Player(Player),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Player {
    Red,
    Blue,
}

impl From<Player> for CellState {
    fn from(player: Player) -> Self {
        CellState::Player(player)
    }
}

impl IntoPropValue<CellState> for Player {
    fn into_prop_value(self) -> CellState {
        self.into()
    }
}

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            CellState::Free => write!(f, "Free"),
            CellState::Player(player) => write!(f, "{}", player),
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Player::Blue => write!(f, "Blue"),
            Player::Red => write!(f, "Red"),
        }
    }
}

pub trait GameTrait {
    fn new(width: u32, height: u32) -> Self;
    fn do_move(&mut self, bar: BarId) -> bool;
    fn restart(&mut self, starting_player: Player);
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn vertical_get(&self, col: u32, row: u32) -> CellState;
    fn horizontal_get(&self, col: u32, row: u32) -> CellState;
    fn cell_get(&self, col: u32, row: u32) -> CellState;
}

pub struct Game<AI: ai::AI> {
    board: BoardState,
    ai: AI,
    ai_player: Player,
}

impl<AI: ai::AI> GameTrait for Game<AI> {
    fn new(width: u32, height: u32) -> Self {
        let board = BoardState::new(width, height);
        let ai = AI::new(Some(&board));
        let ai_player = Player::Blue;
        Self { board , ai, ai_player }
    }

    fn do_move(&mut self, bar: BarId) -> bool {
        let player_move_success = self.board.do_move(bar);
        if !player_move_success {
            return false;
        }
        while self.board.cur_turn == self.ai_player {
            let ai_move = self.ai.next_move(&self.board);
            let ai_move_success = if let Some(ai_move) = ai_move {
                self.board.do_move(ai_move)
            } else {
                false
            };
            if !ai_move_success {
                console::error_1(&format!("AI move failed: {:?}", ai_move).into());
                break;
            }
        }
        true
    }

    fn restart(&mut self, starting_player: Player) {
        self.board.restart(starting_player)
    }

    fn get_width(&self) -> u32 {
        self.board.width
    }

    fn get_height(&self) -> u32 {
        self.board.height
    }
    fn vertical_get(&self, col: u32, row: u32) -> CellState {
        self.board.vstates.get(col, row)
    }

    fn horizontal_get(&self, col: u32, row: u32) -> CellState {
        self.board.hstates.get(col, row)
    }

    fn cell_get(&self, col: u32, row: u32) -> CellState {
        self.board.cell_get(col, row)
    }
}

#[derive(Clone)]
struct BarVec {
    width: u32,
    height: u32,
    direction: BarDirection,
    vec: Vec<CellState>,
}

struct BarVecIdIterator<'a> {
    direction: BarDirection,
    width: u32,
    length: u32,
    cur_index: u32,
    vec: &'a [CellState],
}

impl Iterator for BarVecIdIterator<'_> {
    type Item = (BarId, CellState);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_index >= self.length {
            None
        } else {
            let id = BarId {
                direction: self.direction,
                col: self.cur_index % self.width,
                row: self.cur_index / self.width,
            };
            let state = self.vec[self.cur_index as usize];
            self.cur_index += 1;
            Some((id, state))
        }
    }
}

impl BarVec {
    fn new(width: u32, height: u32, direction: BarDirection) -> Self {
        let vec = vec![CellState::Free; (width * height) as usize];
        Self {
            width,
            height,
            direction,
            vec,
        }
    }

    fn get(&self, col: u32, row: u32) -> CellState {
        self.vec[(row * self.width + col) as usize]
    }

    fn set(&mut self, col: u32, row: u32, state: CellState) {
        self.vec[(row * self.width + col) as usize] = state;
    }

    fn clear(&mut self) {
        for state in self.vec.iter_mut() {
            *state = CellState::Free;
        }
    }

    fn iter(&self) -> BarVecIdIterator {
        BarVecIdIterator {
            direction: self.direction,
            width: self.width,
            length: self.width * self.height,
            cur_index: 0,
            vec: &self.vec,
        }
    }
}

#[derive(Clone)]
pub struct BoardState {
    width: u32,
    height: u32,
    cur_turn: Player,
    vstates: BarVec,
    hstates: BarVec,
    cellstates: Vec<CellState>,
}

impl BoardState {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            cur_turn: Player::Red,
            vstates: BarVec::new(width, height - 1, BarDirection::Vertical),
            hstates: BarVec::new(width - 1, height, BarDirection::Horizontal),
            cellstates: vec![CellState::Free; ((width - 1) * (height - 1)) as usize],
        }
    }

    fn do_move(&mut self, bar: BarId) -> bool {
        let cur_turn = self.cur_turn;
        let neighbors = self.bar_neighbors(bar);
        if self.bar_get(bar) == CellState::Free {
            self.bar_set(bar, cur_turn.into());
            let mut point_gained = false;
            for (neighbor_col, neighbor_row) in neighbors {
                if self.cell_is_full(neighbor_col, neighbor_row) {
                    point_gained = true;
                    self.cell_set(neighbor_col, neighbor_row, cur_turn.into());
                }
            }
            if !point_gained {
                self.cur_turn = match self.cur_turn {
                    Player::Blue => Player::Red,
                    Player::Red => Player::Blue,
                };
            }
            true
        } else {
            false
        }
    }

    fn restart(&mut self, starting_player: Player) {
        for state in &mut self.cellstates {
            *state = CellState::Free;
        }
        self.vstates.clear();
        self.hstates.clear();
        self.cur_turn = starting_player;
    }

    fn cell_get(&self, col: u32, row: u32) -> CellState {
        self.cellstates[(row * (self.width - 1) + col) as usize]
    }

    fn cell_set(&mut self, col: u32, row: u32, state: CellState) {
        self.cellstates[(row * (self.width - 1) + col) as usize] = state;
    }

    fn cell_is_full(&self, col: u32, row: u32) -> bool {
        self.vstates.get(col, row) != CellState::Free
            && self.vstates.get(col + 1, row) != CellState::Free
            && self.hstates.get(col, row) != CellState::Free
            && self.hstates.get(col, row + 1) != CellState::Free
    }

    fn bar_get(&self, bar: BarId) -> CellState {
        let bar_vec = match bar.direction {
            BarDirection::Vertical => &self.vstates,
            BarDirection::Horizontal => &self.hstates,
        };
        bar_vec.get(bar.col, bar.row)
    }

    fn bar_set(&mut self, bar: BarId, state: CellState) {
        let bar_vec = match bar.direction {
            BarDirection::Vertical => &mut self.vstates,
            BarDirection::Horizontal => &mut self.hstates,
        };
        bar_vec.set(bar.col, bar.row, state);
    }

    fn bar_neighbors(&self, bar: BarId) -> Vec<(u32, u32)> {
        match bar.direction {
            BarDirection::Vertical => {
                let mut res = vec![];
                if bar.col != 0 {
                    res.push((bar.col - 1, bar.row));
                }
                if bar.col < self.width - 1 {
                    res.push((bar.col, bar.row));
                }
                res
            }
            BarDirection::Horizontal => {
                let mut res = vec![];
                if bar.row != 0 {
                    res.push((bar.col, bar.row - 1));
                }
                if bar.row < self.height - 1 {
                    res.push((bar.col, bar.row));
                }
                res
            }
        }
    }
}
