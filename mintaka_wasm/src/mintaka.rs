use crate::rusty_renju::{BoardWorker, History, PosWorker};
use crate::{impl_wrapper, to_js_result, to_js_value, try_from_js_value, WebClock};
use mintaka::protocol::response::ResponseSender;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
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

impl_wrapper! {
    pub GameStateWorker { inner: mintaka::game_state::GameState } <-> GameState
}

#[wasm_bindgen(js_name = defaultConfig)]
pub fn default_config() -> Config {
    to_js_value(&mintaka::config::Config::default())
}

#[wasm_bindgen]
impl GameStateWorker {

    #[wasm_bindgen(js_name = default)]
    pub fn default_value() -> Self {
        mintaka::game_state::GameState::default().into()
    }

    #[wasm_bindgen(js_name = fromBoard)]
    pub fn from_board(board: &BoardWorker) -> Self {
        Self {
            inner: board.inner.into(),
        }
    }

    #[wasm_bindgen(js_name = fromHistory)]
    pub fn from_history(history: History) -> Self {
        let history: rusty_renju::history::History = try_from_js_value(history).unwrap();

        Self {
            inner: history.into(),
        }
    }

    pub fn board(&self) -> BoardWorker {
        BoardWorker { inner: self.inner.board }
    }

    pub fn history(&self) -> History {
        serde_wasm_bindgen::to_value(&self.inner.history).unwrap().into()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn play(self, pos: &PosWorker) -> Self {
        self.inner.play((*pos).into()).into()
    }

    pub fn pass(self) -> Self {
        self.inner.pass().into()
    }

    pub fn undo(self) -> Self {
        self.inner.undo_rebuild().into()
    }

    #[wasm_bindgen(js_name = playMut)]
    pub fn play_mut(&mut self, pos: &PosWorker) {
        self.inner.set_mut((*pos).into())
    }

    #[wasm_bindgen(js_name = passMut)]
    pub fn pass_mut(&mut self) {
        self.inner.pass_mut()
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
        let state: mintaka::game_state::GameState = try_from_js_value(state).unwrap_or_default();

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

    pub fn abort(&self) {
        self.inner.store(true, Ordering::Relaxed);
    }

}
