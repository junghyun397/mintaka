use crate::{impl_wrapper, to_js_err, to_js_result, to_js_value, try_from_js_value};
use std::str::FromStr;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsError, JsValue};

impl_wrapper! {
    pub PosWorker { inner: rusty_renju::notation::pos::Pos }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "History")]
    pub type History;

    #[wasm_bindgen(typescript_type = "Color")]
    pub type Color;

    #[wasm_bindgen(typescript_type = "Board")]
    pub type Board;
}

#[wasm_bindgen]
impl PosWorker {

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

impl_wrapper! {
    pub BoardWorker { inner: rusty_renju::board::Board }
}

#[wasm_bindgen]
impl BoardWorker {

    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: rusty_renju::board::Board::default(),
        }
    }

    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(source: &str) -> Result<Self, JsError> {
        let board = rusty_renju::board::Board::from_str(source).map_err(JsError::new)?;
        Ok(Self { inner: board })
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    #[wasm_bindgen(js_name = playerColor)]
    pub fn player_color(&self) -> JsValue {
        to_js_value(&self.inner.player_color)
    }

    pub fn stones(&self) -> u8 {
        self.inner.stones
    }

    #[wasm_bindgen(js_name = isPosEmpty)]
    pub fn is_pos_empty(&self, pos: &PosWorker) -> bool {
        self.inner.is_pos_empty((*pos).into())
    }

    #[wasm_bindgen(js_name = isLegalMove)]
    pub fn is_legal_move(&self, pos: &PosWorker) -> bool {
        self.inner.is_legal_move((*pos).into())
    }

    #[wasm_bindgen(js_name = stoneKind)]
    pub fn stone_kind(&self, pos: &PosWorker) -> Option<JsValue> {
        self.inner.stone_kind((*pos).into())
            .as_ref()
            .map(to_js_value)
    }

    pub fn set(self, pos: &PosWorker) -> Self {
        self.inner.set((*pos).into()).into()
    }

    pub fn unset(self, pos: &PosWorker) -> Self {
        self.inner.unset((*pos).into()).into()
    }

    pub fn pass(&self) -> Self {
        self.inner.pass().into()
    }

    #[wasm_bindgen(js_name = setMut)]
    pub fn set_mut(&mut self, pos: &PosWorker) {
        self.inner.set_mut((*pos).into())
    }

    #[wasm_bindgen(js_name = unsetMut)]
    pub fn unset_mut(&mut self, pos: &PosWorker) {
        self.inner.unset_mut((*pos).into())
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
        Ok(Self { inner: try_from_js_value(value)? })
    }

}
