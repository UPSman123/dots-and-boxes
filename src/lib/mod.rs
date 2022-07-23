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
}

impl<AI: ai::AI> GameTrait for Game<AI> {
    fn new(width: u32, height: u32) -> Self {
        let board = BoardState::new(width, height);
        let ai = AI::new(Some(&board));
        Self { board , ai }
    }

    fn do_move(&mut self, bar: BarId) -> bool {
        let player_move_success = self.board.do_move(bar);
        if player_move_success == false {
            return false;
        }
        let ai_move = self.ai.next_move(&self.board);
        let ai_move_success = if let Some(ai_move) = ai_move {
            self.board.do_move(ai_move)
        } else {
            false
        };
        if !ai_move_success {
            console::error_1(&format!("AI move failed: {:?}", ai_move).into());
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
        self.board.vertical_get(col, row)
    }

    fn horizontal_get(&self, col: u32, row: u32) -> CellState {
        self.board.horizontal_get(col, row)
    }

    fn cell_get(&self, col: u32, row: u32) -> CellState {
        self.board.cell_get(col, row)
    }
}

#[derive(Clone)]
pub struct BoardState {
    width: u32,
    height: u32,
    cur_turn: Player,
    vstates: Vec<CellState>,
    hstates: Vec<CellState>,
    cellstates: Vec<CellState>,
}

impl BoardState {
    fn new(width: u32, height: u32) -> Self {
        let width = width;
        let height = height;
        let vstates = vec![CellState::Free; (width * (height - 1)) as usize];
        let hstates = vec![CellState::Free; ((width - 1) * height) as usize];
        let cellstates = vec![CellState::Free; ((width - 1) * (height - 1)) as usize];
        Self {
            width,
            height,
            cur_turn: Player::Red,
            vstates,
            hstates,
            cellstates,
        }
    }

    fn do_move(&mut self, bar: BarId) -> bool {
        let cur_turn = self.cur_turn;
        let neighbors = self.bar_neighbors(bar);
        let ptr = self.bar_get_mut(bar);
        if *ptr == CellState::Free {
            *ptr = cur_turn.into();
            let mut point_gained = false;
            for (neighbor_col, neighbor_row) in neighbors {
                if self.cell_is_full(neighbor_col, neighbor_row) {
                    point_gained = true;
                    *self.cell_get_mut(neighbor_col, neighbor_row) = cur_turn.into();
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
        let vecs = [&mut self.vstates, &mut self.hstates, &mut self.cellstates];
        for vec in vecs {
            for state in vec {
                *state = CellState::Free;
            }
        }
        self.cur_turn = starting_player;
    }

    fn vertical_get(&self, col: u32, row: u32) -> CellState {
        self.vstates[(row * self.width + col) as usize]
    }

    fn horizontal_get(&self, col: u32, row: u32) -> CellState {
        self.hstates[(row * (self.width - 1) + col) as usize]
    }

    fn cell_get(&self, col: u32, row: u32) -> CellState {
        self.cellstates[(row * (self.width - 1) + col) as usize]
    }
}

impl BoardState {
    fn bar_get_mut(&mut self, bar: BarId) -> &mut CellState {
        match bar.direction {
            BarDirection::Vertical => &mut self.vstates[(bar.row * self.width + bar.col) as usize],
            BarDirection::Horizontal => {
                &mut self.hstates[(bar.row * (self.width - 1) + bar.col) as usize]
            }
        }
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

    fn cell_get_mut(&mut self, col: u32, row: u32) -> &mut CellState {
        &mut self.cellstates[(row * (self.width - 1) + col) as usize]
    }

    fn cell_is_full(&self, col: u32, row: u32) -> bool {
        self.vertical_get(col, row) != CellState::Free
            && self.vertical_get(col + 1, row) != CellState::Free
            && self.horizontal_get(col, row) != CellState::Free
            && self.horizontal_get(col, row + 1) != CellState::Free
    }
}
