use crate::{impl_wrapper, to_js_value, try_from_js_value};
use std::str::FromStr;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsError;
use rusty_renju::dispatch_any_board;

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
    
    #[wasm_bindgen(typescript_type = "RuleKind")]
    pub type RuleKind;

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
pub fn default_board(rule_kind: RuleKind) -> Board {
    let rule_kind = try_from_js_value(rule_kind).unwrap();

    to_js_value(&rusty_renju::board_io::AnyBoard::empty(rule_kind))
}

#[wasm_bindgen(js_name = emptyHash)]
pub fn empty_hash() -> HashKey {
    to_js_value(&rusty_renju::hash_key::HashKey::EMPTY)
}

#[wasm_bindgen(js_name = calculateWinRate)]
pub fn calculate_win_rate(score: Score) -> f32 {
    let score: rusty_renju::notation::score::Score = try_from_js_value(score).unwrap();

    rusty_renju::win_rate::calculate_win_rate(score)
}

impl_wrapper! {
    pub BoardWorker { inner: rusty_renju::board_io::AnyBoard } <-> Board
}

#[wasm_bindgen]
impl BoardWorker {

    #[wasm_bindgen(js_name = fromHistory)]
    pub fn from_history(source: History, rule_kind: RuleKind) -> Result<Self, JsError> {
        let rule_kind = try_from_js_value(rule_kind)?;
        let history: rusty_renju::history::History = try_from_js_value(source)?;

        let board = dispatch_any_board!(wrap rule_kind, (&history).into());

        Ok(Self { inner: board })
    }

    #[wasm_bindgen(js_name = fromString)]
    pub fn from_string(source: &str, rule_kind: RuleKind) -> Result<Self, JsError> {
        let rule_kind = try_from_js_value(rule_kind)?;

        let board = dispatch_any_board!(wrap rule_kind,
            rusty_renju::board::Board::from_str(source).map_err(JsError::new)?
        );

        Ok(Self { inner: board })
    }

    #[wasm_bindgen]
    pub fn empty(rule_kind: RuleKind) -> Self {
        let rule_kind = try_from_js_value(rule_kind).unwrap();

        Self { inner: rusty_renju::board_io::AnyBoard::empty(rule_kind) }
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        dispatch_any_board!(self.inner, board => board.to_string())
    }

    #[wasm_bindgen(js_name = hashKey)]
    pub fn hash_key(&self) -> HashKey {
        let hash_key = dispatch_any_board!(self.inner, board => board.hash_key);

        to_js_value(&hash_key)
    }

    #[wasm_bindgen(js_name = playerColor)]
    pub fn player_color(&self) -> Color {
        let player_color = dispatch_any_board!(self.inner, board => board.player_color);

        to_js_value(&player_color)
    }

    pub fn stones(&self) -> u8 {
        dispatch_any_board!(self.inner, board => board.stones)
    }

    pub fn describe(&self) -> BoardDescribe {
        let describe = dispatch_any_board!(self.inner, board => board.describe());

        to_js_value(&describe)
    }

    #[wasm_bindgen(js_name = isLegalMove)]
    pub fn is_legal_move(&self, pos: Pos) -> bool {
        let pos = try_from_js_value(pos).unwrap();

        dispatch_any_board!(self.inner, board => board.is_legal_move(pos))
    }

    #[wasm_bindgen(js_name = stoneKind)]
    pub fn stone_kind(&self, pos: Pos) -> Option<Color> {
        let pos = try_from_js_value(pos).unwrap();

        dispatch_any_board!(self.inner, board => board
            .stone_kind(pos)
            .as_ref()
            .map(to_js_value)
        )
    }

    #[wasm_bindgen(js_name = setMut)]
    pub fn set_mut(&mut self, action: MaybePos) {
        let maybe_pos: rusty_renju::notation::pos::MaybePos = try_from_js_value(action).unwrap();

        dispatch_any_board!(&mut self.inner, board => {
            if let Some(pos) = maybe_pos.ok() {
                board.set_mut(pos);
            } else {
                board.pass_mut();
            }
        });
    }

    pub fn set(&self, action: MaybePos) -> Self {
        let maybe_pos: rusty_renju::notation::pos::MaybePos = try_from_js_value(action).unwrap();

        dispatch_any_board!(wrap self.inner, board => {
            if let Some(pos) = maybe_pos.ok() {
                board.clone().set(pos)
            } else {
                board.clone().pass()
            }
        }).into()
    }

    #[wasm_bindgen(js_name = unsetMut)]
    pub fn unset_mut(&mut self, action: MaybePos) {
        let maybe_pos: rusty_renju::notation::pos::MaybePos = try_from_js_value(action).unwrap();

        dispatch_any_board!(&mut self.inner, board => {
            if let Some(pos) = maybe_pos.ok() {
                board.unset_mut(pos);
            } else {
                board.unpass_mut();
            }
        });
    }

    pub fn unset(&self, action: MaybePos) -> Self {
        let maybe_pos: rusty_renju::notation::pos::MaybePos = try_from_js_value(action).unwrap();

        dispatch_any_board!(wrap self.inner, board => {
            if let Some(pos) = maybe_pos.ok() {
                board.clone().unset(pos)
            } else {
                board.clone().pass()
            }
        }).into()
    }

}
