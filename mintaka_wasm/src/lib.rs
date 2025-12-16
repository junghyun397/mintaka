pub mod notation;
pub mod rusty_renju;
pub mod mintaka;

use ::mintaka::utils::time::MonotonicClock;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

fn to_js_err(msg: impl ToString) -> JsError {
    JsError::new(&msg.to_string()).into()
}

fn to_js_result(value: &impl Serialize) -> Result<JsValue, JsError> {
    serde_wasm_bindgen::to_value(value).map_err(to_js_err)
}

fn from_js_value<T: DeserializeOwned>(value: JsValue) -> Result<T, JsError> {
    serde_wasm_bindgen::from_value(value).map_err(to_js_err)
}

#[derive(Clone, Copy)]
pub struct WebClock(f64);

impl MonotonicClock for WebClock {
    fn now() -> Self {
        let global = js_sys::global().unchecked_into::<web_sys::WorkerGlobalScope>();
        let performance = global.performance().unwrap();

        Self(performance.now())
    }

    fn elapsed_since(&self, start: Self) -> Duration {
        let now = Self::now();

        Duration::from_secs_f64(now.0 - start.0)
    }
}

#[macro_export] macro_rules! impl_wrapper {
    (pub $wrapper:ident { inner: $inner:path }) => {
        #[wasm_bindgen]
        #[derive(Clone, Copy)]
        pub struct $wrapper {
            pub(crate) inner: $inner,
        }

        impl From<$inner> for $wrapper {
            fn from(inner: $inner) -> Self {
                Self { inner: inner }
            }
        }

        impl From<$wrapper> for $inner {
            fn from(wrapper: $wrapper) -> Self {
                wrapper.inner
            }
        }
    };
    (pub enum $wrapper:ident { inner: $inner:path {$($variant:ident),* $(,)?} }) => {
        #[wasm_bindgen]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $wrapper {
            $($variant),*
        }

        impl From<$inner> for $wrapper {
            fn from(value: $inner) -> Self {
                match value {
                    $(<$inner>::$variant => $wrapper::$variant,)*
                }
            }
        }

        impl From<$wrapper> for $inner {
            fn from(value: $wrapper) -> Self {
                match value {
                    $($wrapper::$variant => <$inner>::$variant,)*
                }
            }
        }
    };
}
