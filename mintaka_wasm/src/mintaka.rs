use crate::rusty_renju::{Board, History, Pos};
use crate::{impl_wrapper, to_js_result, to_js_value, try_from_js_value, WebClock};
use mintaka::protocol::response::ResponseSender;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsError};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Config")]
    pub type Config;

    #[wasm_bindgen(typescript_type = "SearchObjective")]
    pub type SearchObjective;

    #[wasm_bindgen(typescript_type = "Command")]
    pub type Command;

    #[wasm_bindgen(typescript_type = "GameResult")]
    pub type GameResult;

    #[wasm_bindgen(typescript_type = "BestMove")]
    pub type BestMove;

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

    #[wasm_bindgen(js_name = toJs)]
    pub fn to_js(&self) -> Result<GameState, JsError> {
        to_js_result(&self.inner).into()
    }

    #[wasm_bindgen(js_name = fromJs)]
    pub fn from_js(value: GameState) -> Result<Self, JsError> {
        Ok(Self { inner: try_from_js_value(value)? })
    }

}

#[wasm_bindgen]
pub struct GameAgent {
    pub(crate) inner: Arc<RefCell<mintaka::game_agent::GameAgent>>,
}

#[wasm_bindgen]
impl GameAgent {

    #[wasm_bindgen(constructor)]
    pub fn new(config: Config, state: GameState) -> Self {
        let config: mintaka::config::Config = try_from_js_value(config).unwrap_or_default();
        let state: mintaka::state::GameState = try_from_js_value(state).unwrap_or_default();

        Self {
            inner: Arc::new(RefCell::new(mintaka::game_agent::GameAgent::from_state(config, state))),
        }
    }

    pub fn state(&self) -> GameStateWorker {
        GameStateWorker {
            inner: self.inner.borrow().state,
        }
    }

    pub fn command(&mut self, command: Command) -> Result<GameResult, JsError> {
        let command: mintaka::protocol::command::Command = try_from_js_value(command)?;

        let maybe_result = self.inner.borrow_mut().command(command)
            .map_err(|e| JsError::new(&format!("{}", e)))?;

        to_js_result(&maybe_result).into()
    }

    #[wasm_bindgen]
    pub fn launch(
        &mut self,
        search_objective: SearchObjective,
        abort_handle: JsAbortHandle,
    ) -> Result<BestMove, JsError> {
        let inner = Arc::clone(&self.inner);
        let search_objective = try_from_js_value(search_objective)?;

        let best_move = inner.borrow_mut().launch::<WebClock>(
            search_objective,
            JsResponseSender,
            abort_handle.inner.clone(),
        );

        to_js_result(&best_move).into()
    }

}

pub struct JsResponseSender;

impl ResponseSender for JsResponseSender {
    fn response(&self, response: mintaka::protocol::response::Response) {
        let global = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();

        global.post_message(&to_js_result(&response).unwrap()).unwrap();
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct JsAbortHandle {
    inner: Arc<AtomicBool>
}

#[wasm_bindgen]
impl JsAbortHandle {

    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: Arc::new(AtomicBool::new(false)) }
    }

    pub fn ptr(&self) -> u32 {
        Arc::as_ptr(&self.inner) as usize as u32
    }

}
