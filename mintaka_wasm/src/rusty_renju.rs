use crate::notation::{Color, Pos};
use crate::{from_js_value, impl_wrapper, to_js_result};
use std::str::FromStr;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsError, JsValue};

impl_wrapper! {
    pub Board { inner: rusty_renju::board::Board }
}

#[wasm_bindgen]
impl Board {

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
    pub fn player_color(&self) -> Color {
        self.inner.player_color.into()
    }

    pub fn stones(&self) -> u8 {
        self.inner.stones
    }

    #[wasm_bindgen(js_name = isPosEmpty)]
    pub fn is_pos_empty(&self, pos: &Pos) -> bool {
        self.inner.is_pos_empty((*pos).into())
    }

    #[wasm_bindgen(js_name = isLegalMove)]
    pub fn is_legal_move(&self, pos: &Pos) -> bool {
        self.inner.is_legal_move((*pos).into())
    }

    #[wasm_bindgen(js_name = stoneKind)]
    pub fn stone_kind(&self, pos: &Pos) -> Option<Color> {
        self.inner.stone_kind((*pos).into()).map(Into::into)
    }

    pub fn set(self, pos: &Pos) -> Self {
        self.inner.set((*pos).into()).into()
    }

    pub fn unset(self, pos: &Pos) -> Self {
        self.inner.unset((*pos).into()).into()
    }

    pub fn pass(&self) -> Self {
        self.inner.pass().into()
    }

    #[wasm_bindgen(js_name = setMut)]
    pub fn set_mut(&mut self, pos: &Pos) {
        self.inner.set_mut((*pos).into())
    }

    #[wasm_bindgen(js_name = unsetMut)]
    pub fn unset_mut(&mut self, pos: &Pos) {
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
        Ok(Self { inner: from_js_value(value)? })
    }

}
