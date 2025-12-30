use crate::rusty_renju::{Board, BoardWorker, History, Pos};
use crate::{impl_wrapper, to_js_value, try_from_js_value};
use std::cmp::Ordering;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsError;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Config")]
    pub type Config;

    #[wasm_bindgen(typescript_type = "GameState")]
    pub type GameState;
}

#[wasm_bindgen(js_name = defaultConfig)]
pub fn default_config() -> Config {
    to_js_value(&mintaka::config::Config::default())
}

#[wasm_bindgen(js_name = defaultGameState)]
pub fn default_game_state() -> GameState {
    to_js_value(&mintaka::state::GameState::default())
}

#[wasm_bindgen(js_name = de)]
pub fn compare_config(a: Config, b: Config) -> isize {
    let a_config: mintaka::config::Config = try_from_js_value(a).unwrap();
    let b_config: mintaka::config::Config = try_from_js_value(b).unwrap();

    match a_config.cmp(&b_config) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1
    }
}

impl_wrapper! {
    pub GameStateWorker { inner: mintaka::state::GameState } <-> GameState
}

#[wasm_bindgen]
impl GameStateWorker {

    #[wasm_bindgen(js_name = default)]
    pub fn default_value() -> Self {
        mintaka::state::GameState::default().into()
    }

    #[wasm_bindgen(js_name = fromBoard)]
    pub fn from_board(board: Board) -> Self {
        let board: rusty_renju::board::Board = try_from_js_value(board).unwrap();

        Self {
            inner: board.into(),
        }
    }

    #[wasm_bindgen(js_name = fromHistory)]
    pub fn from_history(history: History) -> Self {
        let history: rusty_renju::history::History = try_from_js_value(history).unwrap();

        Self {
            inner: history.into(),
        }
    }

    pub fn board(&self) -> Board {
        to_js_value(&self.inner.board)
    }

    #[wasm_bindgen(js_name = boardWorker)]
    pub fn board_worker(&self) -> BoardWorker {
        BoardWorker { inner: self.inner.board.clone() }
    }

    pub fn history(&self) -> History {
        to_js_value(&self.inner.history)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn play(mut self, pos: Pos) -> Self {
        self.play_mut(pos);
        self
    }

    pub fn pass(mut self) -> Self {
        self.pass_mut();
        self
    }

    pub fn undo(mut self) -> Self {
        self.undo_mut();
        self
    }

    #[wasm_bindgen(js_name = playMut)]
    pub fn play_mut(&mut self, pos: Pos) {
        self.inner.set_mut(try_from_js_value(pos).unwrap());
    }

    #[wasm_bindgen(js_name = passMut)]
    pub fn pass_mut(&mut self) {
        self.inner.pass_mut()
    }

    pub fn undo_mut(&mut self) {
        self.inner.undo_rebuild_mut()
    }

}
