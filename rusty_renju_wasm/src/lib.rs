pub mod rusty_renju;

use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const MINTAKA_TYPES: &'static str = include_str!("../types_ts");

#[wasm_bindgen(typescript_custom_section)]
const UNION_TYPES: &'static str = include_str!("../union_types_ts");

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "panic_hook")]
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

#[macro_export] macro_rules! impl_wrapper {
    (pub $wrapper:ident { inner: $inner:path } <-> $ts_type:ty) => {
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

        #[wasm_bindgen]
        impl $wrapper {

            #[wasm_bindgen(constructor)]
            pub fn new(value: $ts_type) -> Result<Self, JsError> {
                Ok(Self { inner: try_from_js_value(value)? })
            }

            pub fn value(&self) -> $ts_type {
                to_js_value(&self.inner)
            }

        }
    };
}
