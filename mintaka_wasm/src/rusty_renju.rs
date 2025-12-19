use crate::{impl_wrapper, to_js_value, try_from_js_value};
use std::str::FromStr;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsError, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Pos")]
    pub type Pos;

    #[wasm_bindgen(typescript_type = "MaybePos")]
    pub type MaybePos;

    #[wasm_bindgen(typescript_type = "History")]
    pub type History;

    #[wasm_bindgen(typescript_type = "Color")]
    pub type Color;

    #[wasm_bindgen(typescript_type = "Board")]
    pub type Board;
}

impl_wrapper! {
    pub PosWorker { inner: rusty_renju::notation::pos::Pos } <-> Pos
}

#[wasm_bindgen]
impl PosWorker {

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
    pub BoardWorker { inner: rusty_renju::board::Board } <-> Board
}

#[wasm_bindgen]
impl BoardWorker {

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
    pub fn player_color(&self) -> Color {
        to_js_value(&self.inner.player_color)
    }

    pub fn stones(&self) -> u8 {
        self.inner.stones
    }

    #[wasm_bindgen(js_name = isPosEmpty)]
    pub fn is_pos_empty(&self, pos: Pos) -> bool {
        self.inner.is_pos_empty(try_from_js_value(pos).unwrap())
    }

    #[wasm_bindgen(js_name = isLegalMove)]
    pub fn is_legal_move(&self, pos: Pos) -> bool {
        self.inner.is_legal_move(try_from_js_value(pos).unwrap())
    }

    #[wasm_bindgen(js_name = stoneKind)]
    pub fn stone_kind(&self, pos: Pos) -> Option<Color> {
        self.inner.stone_kind(try_from_js_value(pos).unwrap())
            .as_ref()
            .map(to_js_value)
    }

    pub fn set(self, pos: Pos) -> Self {
        self.inner.set(try_from_js_value(pos).unwrap()).into()
    }

    pub fn unset(self, pos: Pos) -> Self {
        self.inner.unset(try_from_js_value(pos).unwrap()).into()
    }

    pub fn pass(&self) -> Self {
        self.inner.pass().into()
    }

    #[wasm_bindgen(js_name = setMut)]
    pub fn set_mut(&mut self, pos: Pos) {
        self.inner.set_mut(try_from_js_value(pos).unwrap())
    }

    #[wasm_bindgen(js_name = unsetMut)]
    pub fn unset_mut(&mut self, pos: Pos) {
        self.inner.unset_mut(try_from_js_value(pos).unwrap())
    }

    #[wasm_bindgen(js_name = passMut)]
    pub fn pass_mut(&mut self) {
        self.inner.pass_mut()
    }

}
