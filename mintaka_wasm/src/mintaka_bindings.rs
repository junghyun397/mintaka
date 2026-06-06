use rusty_renju::utils::empty::Empty;
use crate::{to_js_err, to_js_value, try_from_js_value, WebClock};
use mintaka::protocol::response::ResponseSender;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Arc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsError};
use rusty_renju::notation::rule::RuleKind;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Config")]
    pub type Config;

    #[wasm_bindgen(typescript_type = "SearchObjective")]
    pub type SearchObjective;

    #[wasm_bindgen(typescript_type = "Timer")]
    pub type Timer;

    #[wasm_bindgen(typescript_type = "Command")]
    pub type Command;

    #[wasm_bindgen(typescript_type = "CommandResult")]
    pub type CommandResult;

    #[wasm_bindgen(typescript_type = "BestMove")]
    pub type BestMove;

    #[wasm_bindgen(typescript_type = "GameState")]
    pub type GameState;

    #[wasm_bindgen(typescript_type = "HashKey")]
    pub type HashKey;
}

#[wasm_bindgen]
pub struct GameAgent {
    config: mintaka::config::Config,
    inner: Arc<RefCell<mintaka::game_agent::GameAgent<{ RuleKind::Renju }>>>,
}

#[wasm_bindgen]
impl GameAgent {
    #[wasm_bindgen(constructor)]
    pub fn new(config: Config, state: GameState) -> Self {
        let config: mintaka::config::Config = try_from_js_value(config).unwrap_or_default();
        let state: mintaka::game_state::GameState<{ RuleKind::Renju }> = try_from_js_value(state).unwrap_or_else(|_| mintaka::game_state::GameState::empty());

        Self {
            config,
            inner: Arc::new(RefCell::new(mintaka::game_agent::GameAgent::from_state(config, state))),
        }
    }

    pub fn config(&mut self, config: Config) {
        let config: mintaka::config::Config = try_from_js_value(config).unwrap_or_default();

        if config.tt_size != self.config.tt_size {
            let _ = self.inner.borrow_mut().command(mintaka::protocol::command::Command::RebuildTT(config.tt_size));
        }

        self.config = config;
    }

    pub fn command(&mut self, command: Command) -> Result<CommandResult, JsError> {
        let command: mintaka::protocol::command::Command = try_from_js_value(command)?;

        let result = self.inner.borrow_mut().command(command)
            .map_err(to_js_err)?;

        Ok(to_js_value(&result))
    }

    #[wasm_bindgen]
    pub fn launch(
        &mut self,
        timer: Timer,
        search_objective: SearchObjective,
        counter_handle: &JsCounterHandle,
        abort_handle: &JsAbortHandle,
    ) -> Result<BestMove, JsError> {
        let inner = Arc::clone(&self.inner);
        let timer = try_from_js_value(timer)?;
        let search_objective = try_from_js_value(search_objective)?;

        let best_move = inner.borrow_mut().launch::<WebClock>(
            self.config,
            timer,
            search_objective,
            JsResponseSender,
            counter_handle.inner.clone(),
            abort_handle.inner.clone(),
        );

        Ok(to_js_value(&best_move))
    }

    #[wasm_bindgen(js_name = "hashKey")]
    pub fn hash_key(&self) -> HashKey {
        to_js_value(&self.inner.borrow().state.board.hash_key)
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
        self.inner.as_ptr() as usize as u32
    }

}

#[wasm_bindgen]
#[derive(Clone)]
pub struct JsCounterHandle {
    inner: Arc<AtomicU32>
}

#[wasm_bindgen]
impl JsCounterHandle {

    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: Arc::new(AtomicU32::new(0)) }
    }

    pub fn ptr(&self) -> u32 {
        self.inner.as_ptr() as usize as u32
    }

}
