use std::fmt;
use web_sys::console;
use yew::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
enum CellState {
    Free,
    Blue,
    Red,
}

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            CellState::Free => write!(f, "Free"),
            CellState::Blue => write!(f, "Blue"),
            CellState::Red => write!(f, "Red"),
        }
    }
}

enum BoardMsg {
    ClickVBar { col: u32, row: u32 },
    ClickHBar { col: u32, row: u32 },
}

struct BoardComp {
    width: u32,
    height: u32,
    vstates: Vec<CellState>,
    hstates: Vec<CellState>,
    cellstates: Vec<CellState>,
    cur_turn: CellState,
}

impl Component for BoardComp {
    type Message = BoardMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let width = 4;
        let height = 3;
        let vstates = vec![CellState::Free; width * (height - 1)];
        let hstates = vec![CellState::Free; (width - 1) * height];
        let cellstates = vec![CellState::Free; (width - 1) * (height - 1)];
        Self {
            width: width.try_into().unwrap(),
            height: height.try_into().unwrap(),
            vstates,
            hstates,
            cellstates,
            cur_turn: CellState::Blue,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! { <div class="center-content"> {
            self.gen_table(ctx)
        } </div>}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        let cur_turn = self.cur_turn;
        let (ptr, neighbors) = match msg {
            BoardMsg::ClickVBar { col, row } => {
                let neighbors = self.vneighbors(col, row);
                (self.vstate_get_mut(col, row), neighbors)
            }
            BoardMsg::ClickHBar { col, row } => {
                let neighbors = self.hneighbors(col, row);
                (self.hstate_get_mut(col, row), neighbors)
            }
        };
        if *ptr == CellState::Free {
            *ptr = cur_turn;
            let mut point_gained = false;
            for (neighbor_col, neighbor_row) in neighbors {
                if self.cell_is_full(neighbor_col, neighbor_row) {
                    point_gained = true;
                    *self.cellstate_get_mut(neighbor_col, neighbor_row) = cur_turn;
                }
            }
            if !point_gained {
                self.cur_turn = match self.cur_turn {
                    CellState::Free => panic!("cur turn is \"Free\""),
                    CellState::Blue => CellState::Red,
                    CellState::Red => CellState::Blue,
                };
            }
            true
        } else {
            false
        }
    }
}

impl BoardComp {
    fn vstate_get(&self, col: u32, row: u32) -> CellState {
        self.vstates[(row * self.width + col) as usize]
    }

    fn vstate_get_mut(&mut self, col: u32, row: u32) -> &mut CellState {
        &mut self.vstates[(row * self.width + col) as usize]
    }

    fn hstate_get(&self, col: u32, row: u32) -> CellState {
        self.hstates[(row * (self.width - 1) + col) as usize]
    }

    fn hstate_get_mut(&mut self, col: u32, row: u32) -> &mut CellState {
        &mut self.hstates[(row * (self.width - 1) + col) as usize]
    }

    fn cellstate_get(&self, col: u32, row: u32) -> CellState {
        self.cellstates[(row * (self.width - 1) + col) as usize]
    }

    fn cellstate_get_mut(&mut self, col: u32, row: u32) -> &mut CellState {
        &mut self.cellstates[(row * (self.width - 1) + col) as usize]
    }

    fn vneighbors(&self, col: u32, row: u32) -> Vec<(u32, u32)> {
        let mut res = vec![];
        if col != 0 {
            res.push((col - 1, row));
        }
        if col < self.width - 1 {
            res.push((col, row));
        }
        res
    }

    fn hneighbors(&self, col: u32, row: u32) -> Vec<(u32, u32)> {
        let mut res = vec![];
        if row != 0 {
            res.push((col, row - 1));
        }
        if row < self.height - 1 {
            res.push((col, row));
        }
        res
    }

    fn cell_is_full(&self, col: u32, row: u32) -> bool {
        self.vstate_get(col, row) != CellState::Free
            && self.vstate_get(col + 1, row) != CellState::Free
            && self.hstate_get(col, row) != CellState::Free
            && self.hstate_get(col, row + 1) != CellState::Free
    }

    fn gen_table(&self, ctx: &Context<Self>) -> Html {
        let span = 4;
        let columns = (self.width - 1) * span + self.width;
        let style = format!("--nr-columns: {}; --span: {}", columns, span);
        let mut rows = vec![];
        for row_id in 0..self.height - 1 {
            rows.push(self.gen_thin_row(ctx, row_id));
            rows.push(self.gen_thick_row(ctx, row_id));
        }
        rows.push(self.gen_thin_row(ctx, self.height - 1));
        html! {
            <div class="board" { style }> { rows.into_iter().collect::<Html>() } </div>
        }
    }

    fn gen_thin_row(&self, ctx: &Context<Self>, row_idx: u32) -> Html {
        let mut cells = vec![];
        for col_idx in 0..self.width - 1 {
            cells.push(Self::gen_dot(col_idx, row_idx));
            cells.push(self.gen_hbar(ctx, col_idx, row_idx));
        }
        cells.push(Self::gen_dot(self.width - 1, row_idx));
        cells.into_iter().collect::<Html>()
    }

    fn gen_thick_row(&self, ctx: &Context<Self>, row_idx: u32) -> Html {
        let mut cells = vec![];
        for col_idx in 0..self.width - 1 {
            cells.push(self.gen_vbar(ctx, col_idx, row_idx));
            cells.push(self.gen_inner_cell(col_idx, row_idx));
        }
        cells.push(self.gen_vbar(ctx, self.width - 1, row_idx));
        cells.into_iter().collect::<Html>()
    }

    fn gen_dot(col: u32, row: u32) -> Html {
        let key = format!("d {} {}", col, row);
        html! { <div
            { key }
            class="dot"
        ></div> }
    }

    fn gen_hbar(&self, ctx: &Context<Self>, col: u32, row: u32) -> Html {
        let link = ctx.link();
        let state = self.hstate_get(col, row).to_string();
        let key = format!("h {} {} {}", state, col, row);
        html! { <div
            { key }
            class="bar hbar"
            data-state={ state }
            onclick={link.callback(move |_| BoardMsg::ClickHBar {col, row})}
        ></div> }
    }

    fn gen_vbar(&self, ctx: &Context<Self>, col: u32, row: u32) -> Html {
        let link = ctx.link();
        let state = self.vstate_get(col, row).to_string();
        let key = format!("v {} {} {}", state, col, row);
        html! { <div
            { key }
            class="bar vbar"
            data-state={ state }
            onclick={link.callback(move |_| BoardMsg::ClickVBar {col, row})}
        ></div> }
    }

    fn gen_inner_cell(&self, col: u32, row: u32) -> Html {
        let state = self.cellstate_get(col, row).to_string();
        let key = format!("c {} {} {}", state, col, row);
        html! { <div
            { key }
            class="inner-cell"
            data-state={ state }
        ></div> }
    }
}

struct AppComp {}

impl Component for AppComp {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        console::log_1(&"page load".into());
        html! {
            <div>
                <h1>{ "Hi this is a page." }</h1>
                <BoardComp/>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<AppComp>();
}
