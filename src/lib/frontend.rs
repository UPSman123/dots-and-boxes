use web_sys::console;
use yew::prelude::*;
use yew::Properties;

use crate::lib::{ai::AIMinMax, *};

pub enum BoardMsg {
    ClickBar {
        direction: BarDirection,
        col: u32,
        row: u32,
    },
    StartGame(Player),
}

#[derive(PartialEq, Properties)]
struct BoardProps {
    width: u32,
    height: u32,
    app_update: Callback<AppMsg>,
}

struct BoardComp<G: GameTrait> {
    board_state: G,
}

impl<G: GameTrait + 'static> Component for BoardComp<G> {
    type Message = BoardMsg;
    type Properties = BoardProps;

    fn create(ctx: &Context<Self>) -> Self {
        let board_update = ctx.link().callback(std::convert::identity);
        ctx.props()
            .app_update
            .emit(AppMsg::BoardUpdate(board_update));
        let board_state = G::new(ctx.props().width, ctx.props().height);
        Self { board_state }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.gen_table(ctx)
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            BoardMsg::ClickBar {
                direction,
                col,
                row,
            } => self.board_state.do_move(BarId {
                direction,
                col,
                row,
            }),
            BoardMsg::StartGame(player) => {
                self.board_state.restart(player);
                true
            }
        }
    }
}

impl<G: GameTrait + 'static> BoardComp<G> {
    fn gen_table(&self, ctx: &Context<Self>) -> Html {
        let span = 4;
        let columns = (self.board_state.get_width() - 1) * span + self.board_state.get_width();
        let style = format!("--nr-columns: {}; --span: {}", columns, span);
        let mut rows = vec![];
        for row_id in 0..self.board_state.get_height() - 1 {
            rows.push(self.gen_thin_row(ctx, row_id));
            rows.push(self.gen_thick_row(ctx, row_id));
        }
        rows.push(self.gen_thin_row(ctx, self.board_state.get_height() - 1));
        html! {
            <div class="board" { style }> { rows.into_iter().collect::<Html>() } </div>
        }
    }

    fn gen_thin_row(&self, ctx: &Context<Self>, row_idx: u32) -> Html {
        let mut cells = vec![];
        for col_idx in 0..self.board_state.get_width() - 1 {
            cells.push(Self::gen_dot(col_idx, row_idx));
            cells.push(self.gen_hbar(ctx, col_idx, row_idx));
        }
        cells.push(Self::gen_dot(self.board_state.get_width() - 1, row_idx));
        cells.into_iter().collect::<Html>()
    }

    fn gen_thick_row(&self, ctx: &Context<Self>, row_idx: u32) -> Html {
        let mut cells = vec![];
        for col_idx in 0..self.board_state.get_width() - 1 {
            cells.push(self.gen_vbar(ctx, col_idx, row_idx));
            cells.push(self.gen_inner_cell(col_idx, row_idx));
        }
        cells.push(self.gen_vbar(ctx, self.board_state.get_width() - 1, row_idx));
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
        let state = self.board_state.horizontal_get(col, row).to_string();
        let key = format!("h {} {} {}", state, col, row);
        let direction = BarDirection::Horizontal;
        html! { <div
            { key }
            class="bar hbar"
            data-state={ state }
            onclick={link.callback(move |_| BoardMsg::ClickBar {direction, col, row})}
        ></div> }
    }

    fn gen_vbar(&self, ctx: &Context<Self>, col: u32, row: u32) -> Html {
        let link = ctx.link();
        let state = self.board_state.vertical_get(col, row).to_string();
        let key = format!("v {} {} {}", state, col, row);
        let direction = BarDirection::Vertical;
        html! { <div
            { key }
            class="bar vbar"
            data-state={ state }
            onclick={link.callback(move |_| BoardMsg::ClickBar {direction, col, row})}
        ></div> }
    }

    fn gen_inner_cell(&self, col: u32, row: u32) -> Html {
        let state = self.board_state.cell_get(col, row).to_string();
        let key = format!("c {} {} {}", state, col, row);
        html! { <div
            { key }
            class="inner-cell"
            data-state={ state }
        ></div> }
    }
}

struct StartButtonComp {}

#[derive(Properties, PartialEq)]
struct StartButtonProps {
    player: Player,
    app_update: Callback<AppMsg>,
}

impl Component for StartButtonComp {
    type Message = ();
    type Properties = StartButtonProps;

    fn create(_ctx: &Context<Self>) -> Self {
        StartButtonComp {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let player = props.player;
        let onclick = props.app_update.reform(move |_| AppMsg::StartGame(player));
        html! {
            <button
                {onclick}
            >{format!("Start ({})", props.player)}</button>
        }
    }
}

#[derive(Properties, PartialEq)]
struct ControlBarProps {
    app_update: Callback<AppMsg>,
}

struct ControlBarComp {}

impl Component for ControlBarComp {
    type Message = ();
    type Properties = ControlBarProps;

    fn create(_ctx: &Context<Self>) -> Self {
        ControlBarComp {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let app_update = ctx.props().app_update.clone();
        html! {
        <div class={"control-bar"}>
            <h2>{"control bar"}</h2>
            <StartButtonComp player={Player::Red} app_update={app_update.clone()}/>
            <StartButtonComp player={Player::Blue} app_update={app_update}/>
        </div>}
    }
}

pub enum AppMsg {
    StartGame(Player),
    BoardUpdate(Callback<BoardMsg>),
}

pub struct AppComp {
    board_update: Option<Callback<BoardMsg>>,
}

impl Component for AppComp {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { board_update: None }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        console::log_1(&"page load".into());
        let app_update = ctx.link().callback(std::convert::identity);
        html! {
            <>
            <h1>{ "Dots and Boxes" }</h1>
            <div class="content">
                <ControlBarComp app_update={app_update.clone()}/>
                <BoardComp<Game<AIMinMax>> width=4 height=4 app_update={app_update.clone()}/>
            </div>
            </>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::StartGame(starting_player) => {
                if let Some(board_update) = &self.board_update {
                    board_update.emit(BoardMsg::StartGame(starting_player));
                    true
                } else {
                    console::error_1(&"didn't get board_update callback".into());
                    false
                }
            }
            AppMsg::BoardUpdate(cb) => {
                self.board_update = Some(cb);
                false
            }
        }
    }
}
