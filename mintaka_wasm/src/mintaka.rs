use crate::notation::Pos;
use crate::rusty_renju::Board;
use crate::{from_js_value, impl_wrapper, to_js_result, WebClock};
use mintaka::config::{Config, SearchObjective as RustSearchObjective};
use mintaka::protocol::command::Command;
use mintaka::protocol::response::ResponseSender;
use serde::Serialize;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsError, JsValue};

impl_wrapper! {
    pub GameState { inner: mintaka::game_state::GameState }
}

#[wasm_bindgen]
impl GameState {

    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: mintaka::game_state::GameState::default(),
        }
    }

    #[wasm_bindgen(js_name = fromBoard)]
    pub fn from_board(board: &Board) -> Self {
        Self {
            inner: board.inner.into(),
        }
    }

    #[wasm_bindgen(js_name = fromHistory)]
    pub fn from_history(history: JsValue) -> Self {
        let history: rusty_renju::history::History = from_js_value(history).unwrap();

        Self {
            inner: history.into(),
        }
    }

    pub fn board(&self) -> Board {
        Board { inner: self.inner.board }
    }

    pub fn history(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.history).unwrap()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn play(self, pos: &Pos) -> Self {
        self.inner.play((*pos).into()).into()
    }

    pub fn pass(self) -> Self {
        self.inner.pass().into()
    }

    pub fn undo(self) -> Self {
        self.inner.undo_rebuild().into()
    }

    #[wasm_bindgen(js_name = playMut)]
    pub fn play_mut(&mut self, pos: &Pos) {
        self.inner.set_mut((*pos).into())
    }

    #[wasm_bindgen(js_name = passMut)]
    pub fn pass_mut(&mut self) {
        self.inner.pass_mut()
    }

    #[wasm_bindgen(js_name = toJs)]
    pub fn to_js(&self) -> Result<JsValue, JsError> {
        to_js_result(&self.inner)
    }

    #[wasm_bindgen(js_name = fromJs)]
    pub fn from_js(value: JsValue) -> Result<Self, JsError> {
        Ok(Self { inner: from_js_value(value)? })
    }

}

impl_wrapper! {
    pub enum SearchObjective { inner: RustSearchObjective { Best, Zeroing, Pondering } }
}

#[wasm_bindgen]
pub struct GameAgent {
    pub(crate) inner: Arc<RefCell<mintaka::game_agent::GameAgent>>,
}

#[wasm_bindgen]
impl GameAgent {

    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Self {
        let config: Config = from_js_value(config).unwrap_or_default();

        Self {
            inner: Arc::new(RefCell::new(mintaka::game_agent::GameAgent::new(config))),
        }
    }

    #[wasm_bindgen(js_name = fromState)]
    pub fn from_state(config: JsValue, state: GameState) -> Self {
        let config: Config = from_js_value(config).unwrap_or_default();

        Self {
            inner: Arc::new(RefCell::new(mintaka::game_agent::GameAgent::from_state(config, state.inner))),
        }
    }

    pub fn state(&self) -> GameState {
        GameState {
            inner: self.inner.borrow().state,
        }
    }

    pub fn command(&mut self, command: JsValue) -> Result<JsValue, JsError> {
        let command: Command = from_js_value(command)?;

        let maybe_result = self.inner.borrow_mut().command(command)
            .map_err(|e| JsError::new(&format!("{}", e)))?;

        to_js_result(&maybe_result)
    }

    #[wasm_bindgen]
    pub fn launch(
        &mut self,
        search_objective: SearchObjective,
        abort_handle: JsAbortHandle,
    ) -> Result<JsValue, JsError> {
        let inner = Arc::clone(&self.inner);

        let search_objective: RustSearchObjective = search_objective.into();

        let best_move = inner.borrow_mut().launch::<WebClock>(
            search_objective,
            JsResponseSender,
            abort_handle.inner.clone(),
        );

        to_js_result(&best_move)
    }

}

#[derive(Serialize)]
struct JsOutputAdaptor {
    #[serde(rename = "type")]
    type_field: String,
    payload: mintaka::protocol::response::Response,
}

pub struct JsResponseSender;

impl ResponseSender for JsResponseSender {
    fn response(&self, response: mintaka::protocol::response::Response) {
        let global = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();

        let js_response = JsOutputAdaptor {
            type_field: "response".to_string(),
            payload: response,
        };

        global.post_message(&to_js_result(&js_response).unwrap()).unwrap();
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

    pub fn abort(&self) {
        self.inner.store(true, Ordering::Relaxed);
    }

}
