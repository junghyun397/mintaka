pub mod mintaka_worker;

pub use wasm_bindgen_rayon::init_thread_pool;

use mintaka::utils::time::MonotonicClock;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsError, JsValue};

#[wasm_bindgen(typescript_custom_section)]
const MINTAKA_TYPES: &'static str = include_str!("../../mintaka_wasm/types_ts");

#[wasm_bindgen(typescript_custom_section)]
const UNION_TYPES: &'static str = include_str!("../../mintaka_wasm/union_types_ts");

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn to_js_err(err: impl ToString) -> JsError {
    JsError::new(&err.to_string()).into()
}

pub fn to_js_value<R: From<JsValue>>(value: &impl Serialize) -> R {
    serde_wasm_bindgen::to_value(value)
        .unwrap()
        .into()
}

pub fn try_from_js_value<R: DeserializeOwned>(value: impl Into<JsValue>) -> Result<R, JsError> {
    serde_wasm_bindgen::from_value(value.into())
        .map_err(to_js_err)
}

#[derive(Clone, Copy)]
pub struct WebClock(f64);

impl MonotonicClock for WebClock {
    fn now() -> Self {
        let performance = js_sys::global()
            .unchecked_into::<web_sys::WorkerGlobalScope>()
            .performance()
            .unwrap();

        Self(performance.time_origin() + performance.now())
    }

    fn elapsed_since(&self, start: Self) -> Duration {
        let delta_ms = (Self::now().0 - start.0).max(0.0);

        Duration::from_secs_f64(delta_ms / 1000.0)
    }
}
