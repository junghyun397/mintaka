use crate::{to_js_result, to_js_value, try_from_js_value, WebClock};
use mintaka::protocol::response::ResponseSender;
use std::cell::RefCell;
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

        let _ = global.post_message(&to_js_value(&response));
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
