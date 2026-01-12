use crate::rusty_renju::Color;
use crate::to_js_value;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = "Score")]
    pub type Score;

    #[wasm_bindgen(typescript_type = "Config")]
    pub type Config;
}

#[wasm_bindgen(js_name = defaultConfig)]
pub fn default_config() -> Config {
    to_js_value(&mintaka::config::Config::default())
}

#[wasm_bindgen(js_name = calculateNormEval)]
pub fn calculate_norm_eval(score: Score, color: Color) -> f32 {
    0.0
}
