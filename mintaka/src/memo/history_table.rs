use crate::value;
use crate::value::Depth;
use core::f64;
use rusty_renju::history::History;
use rusty_renju::notation::color::{Color, ColorContainer};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos, PosList};
use serde::{Deserialize, Serialize, Serializer};

pub type QuietPlied = PosList<{ 248 }>;
pub type TacticalPlied = PosList<{ 56 }>;

pub const MAX_HISTORY_SCORE: i16 = i16::MAX / 2;

#[derive(Copy, Clone)]
pub struct HistoryTable {
    pub quiet: ColorContainer<[i16; pos::BOARD_SIZE]>,
    pub three: ColorContainer<[i16; pos::BOARD_SIZE]>,
    pub four: ColorContainer<[i16; pos::BOARD_SIZE]>,
    pub counter: ColorContainer<[MaybePos; pos::BOARD_SIZE]>,
}

impl HistoryTable {

    pub fn new() -> Self {
        Self {
            quiet: ColorContainer::new([0; pos::BOARD_SIZE], [0; pos::BOARD_SIZE]),
            three: ColorContainer::new([0; pos::BOARD_SIZE], [0; pos::BOARD_SIZE]),
            four: ColorContainer::new([0; pos::BOARD_SIZE], [0; pos::BOARD_SIZE]),
            counter: ColorContainer::new([MaybePos::NONE; pos::BOARD_SIZE], [MaybePos::NONE; pos::BOARD_SIZE]),
        }
    }

    pub fn update_quiet(&mut self, history: &History, quiet_plied: QuietPlied, color: Color, best_move: Pos, depth: Depth) {
        for &pos in quiet_plied.iter() {
            let bonus = (depth * depth) as i16 * ((pos == best_move) as i16 * 2 - 1);

            let updated_score = self.quiet[color][pos.idx_usize()] + bonus;
            self.quiet[color][pos.idx_usize()] = updated_score.clamp(-MAX_HISTORY_SCORE, MAX_HISTORY_SCORE);

            if history.len() != 0
                && let Some(prev_move) = history.recent_action().ok()
            {
                self.counter[color][prev_move.idx_usize()] = best_move.into();
            }
        }
    }

    pub fn update_tactical(&mut self, three_plied: TacticalPlied, four_plied: TacticalPlied, color: Color, best_move: Pos, depth: Depth) {
        for &pos in three_plied.iter() {
            let bonus = (depth * depth) as i16 * ((pos == best_move) as i16 * 2 - 1);

            let updated_score = self.three[color][pos.idx_usize()] + bonus;
            self.three[color][pos.idx_usize()] = updated_score.clamp(-MAX_HISTORY_SCORE, MAX_HISTORY_SCORE);
        }

        for &pos in four_plied.iter() {
            let bonus = (depth * depth) as i16 * ((pos == best_move) as i16 * 2 - 1);

            let updated_score = self.four[color][pos.idx_usize()] + bonus;
            self.four[color][pos.idx_usize()] = updated_score.clamp(-MAX_HISTORY_SCORE, MAX_HISTORY_SCORE);
        }
    }

    pub fn increase_age(&mut self) {
        fn ageing_score(score: &mut i16) {
            *score = (*score as f64 * value::HISTORY_TABLE_AGEING_MUL) as i16;
        }

        self.quiet[Color::Black].iter_mut().for_each(ageing_score);
        self.quiet[Color::White].iter_mut().for_each(ageing_score);

        self.three[Color::Black].iter_mut().for_each(ageing_score);
        self.three[Color::White].iter_mut().for_each(ageing_score);
    }

}

#[derive(Serialize, Deserialize)]
struct HistoryTableData {
    #[serde(
        serialize_with = "crate::utils::serde::serialize_array",
        deserialize_with = "crate::utils::serde::deserialize_array"
    )]
    quiet: [u8; 2 * pos::BOARD_SIZE * 2],
    #[serde(
        serialize_with = "crate::utils::serde::serialize_array",
        deserialize_with = "crate::utils::serde::deserialize_array"
    )]
    three: [u8; 2 * pos::BOARD_SIZE * 2],
    #[serde(
        serialize_with = "crate::utils::serde::serialize_array",
        deserialize_with = "crate::utils::serde::deserialize_array"
    )]
    four: [u8; 2 * pos::BOARD_SIZE * 2],
    #[serde(
        serialize_with = "crate::utils::serde::serialize_array",
        deserialize_with = "crate::utils::serde::deserialize_array"
    )]
    counter: [u8; pos::BOARD_SIZE * 2],
}

impl Serialize for HistoryTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let data = unsafe { HistoryTableData {
            quiet: std::mem::transmute(self.quiet),
            three: std::mem::transmute(self.three),
            four: std::mem::transmute(self.four),
            counter: std::mem::transmute(self.counter),
        } };

        data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HistoryTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let data = HistoryTableData::deserialize(deserializer)?;

        unsafe { Ok(Self {
            quiet: std::mem::transmute(data.quiet),
            three: std::mem::transmute(data.quiet),
            four: std::mem::transmute(data.four),
            counter: std::mem::transmute(data.counter),
        }) }
    }
}
