use crate::{impl_wrapper, to_js_value, try_from_js_value};
use std::str::FromStr;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsError;

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

    #[wasm_bindgen(typescript_type = "HashKey")]
    pub type HashKey;

    #[wasm_bindgen(typescript_type = "BoardDescribe")]
    pub type BoardDescribe;

    #[wasm_bindgen(typescript_type = "Score")]
    pub type Score;
}

#[wasm_bindgen(js_name = defaultBoard)]
pub fn default_board() -> Board {
    to_js_value(&rusty_renju::board::Board::default())
}

#[wasm_bindgen(js_name = emptyHash)]
pub fn empty_hash() -> HashKey {
    to_js_value(&rusty_renju::memo::hash_key::HashKey::EMPTY)
}

#[wasm_bindgen(js_name = calculateWinRate)]
pub fn calculate_win_rate(score: Score) -> f32 {
    let score: rusty_renju::notation::score::Score = try_from_js_value(score).unwrap();

    rusty_renju::win_rate::calculate_win_rate(score)
}

impl_wrapper! {
    pub BoardWorker { inner: rusty_renju::board::Board } <-> Board
}

#[wasm_bindgen]
impl BoardWorker {

    #[wasm_bindgen(js_name = fromHistory)]
    pub fn from_history(source: History) -> Result<Self, JsError> {
        let history: rusty_renju::history::History = try_from_js_value(source)?;

        Ok(Self { inner: (&history).into() })
    }

    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(source: &str) -> Result<Self, JsError> {
        let board = rusty_renju::board::Board::from_str(source).map_err(JsError::new)?;
        Ok(Self { inner: board })
    }

    #[wasm_bindgen]
    pub fn empty() -> Self {
        Self { inner: rusty_renju::board::Board::EMPTY_BOARD }
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    #[wasm_bindgen(js_name = hashKey)]
    pub fn hash_key(&self) -> HashKey {
        to_js_value(&self.inner.hash_key)
    }

    #[wasm_bindgen(js_name = playerColor)]
    pub fn player_color(&self) -> Color {
        to_js_value(&self.inner.player_color)
    }

    pub fn stones(&self) -> u8 {
        self.inner.stones
    }

    pub fn pattern(&self, color: Color, pos: Pos) -> u32 {
        let color: rusty_renju::notation::color::Color = try_from_js_value(color).unwrap();
        let pos: rusty_renju::notation::pos::Pos = try_from_js_value(pos).unwrap();

        self.inner.patterns.field[color][pos.idx_usize()].into()
    }

    pub fn describe(&self, history: &History) -> BoardDescribe {
        let history: rusty_renju::history::History = try_from_js_value(history).unwrap();

        to_js_value(&self.inner.describe(&history))
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

    pub fn set(self, pos: MaybePos) -> Self {
        let maybe_pos: rusty_renju::notation::pos::MaybePos = try_from_js_value(pos).unwrap();

        if maybe_pos.is_some() {
            self.inner.set(maybe_pos.unwrap()).into()
        } else {
            self.inner.pass().into()
        }
    }

    pub fn unset(self, pos: MaybePos) -> Self {
        let maybe_pos: rusty_renju::notation::pos::MaybePos = try_from_js_value(pos).unwrap();

        if maybe_pos.is_some() {
            self.inner.unset(maybe_pos.unwrap()).into()
        } else {
            self.inner.pass().into()
        }
    }

}
