use crate::{impl_wrapper, to_js_err};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsError, JsValue};

impl_wrapper! {
    pub enum Color { inner: rusty_renju::notation::color::Color { Black, White } }
}

impl_wrapper! {
    pub enum RuleKind { inner: rusty_renju::notation::rule::RuleKind { Gomoku, Renju } }
}

impl_wrapper! {
    pub Pos { inner: rusty_renju::notation::pos::Pos }
}

#[wasm_bindgen]
impl Pos {

    #[wasm_bindgen(constructor)]
    pub fn new(value: JsValue) -> Result<Self, JsError> {
        if value.is_string() {
            let pos: rusty_renju::notation::pos::Pos = value.as_string().unwrap().parse().map_err(to_js_err)?;

            Ok(pos.into())
        } else {
            Err(JsError::new("invalid argument"))
        }
    }

    #[wasm_bindgen(js_name = fromIndex)]
    pub fn from_index(idx: u8) -> Self {
        rusty_renju::notation::pos::Pos::from_index(idx).into()
    }

    #[wasm_bindgen(js_name = fromCartesian)]
    pub fn from_cartesian(row: u8, col: u8) -> Self {
        rusty_renju::notation::pos::Pos::from_cartesian(row, col).into()
    }

    pub fn idx(&self) -> u8 {
        self.inner.idx()
    }

    pub fn row(&self) -> u8 {
        self.inner.row()
    }

    pub fn col(&self) -> u8 {
        self.inner.col()
    }

    #[wasm_bindgen(js_name = toCartesian)]
    pub fn to_cartesian(&self) -> js_sys::Array {
        let (r, c) = self.inner.to_cartesian();
        [JsValue::from(r), JsValue::from(c)].into_iter().collect()
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

}
