use crate::params;
use crate::value::Depth;
use core::f64;
use rusty_renju::history::History;
use rusty_renju::notation::color::{Color, ColorContainer};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos, PosList};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, Serializer};

pub type QuietPlied = PosList<{ 256 - 8 }>;
pub type TacticalPlied = PosList<{ 64 - 8 }>;

pub const MAX_HISTORY_SCORE: i32 = (i16::MAX / 2) as i32;

#[derive(Copy, Clone)]
pub struct HistoryTable {
    pub quiet: ColorContainer<[i16; pos::BOARD_SIZE]>,
    pub three: ColorContainer<[i16; pos::BOARD_SIZE]>,
    pub four: ColorContainer<[i16; pos::BOARD_SIZE]>,
    pub counter: ColorContainer<[MaybePos; pos::BOARD_SIZE]>,
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl HistoryTable {

    pub const EMPTY: Self = Self {
        quiet: ColorContainer::new([0; pos::BOARD_SIZE], [0; pos::BOARD_SIZE]),
        three: ColorContainer::new([0; pos::BOARD_SIZE], [0; pos::BOARD_SIZE]),
        four: ColorContainer::new([0; pos::BOARD_SIZE], [0; pos::BOARD_SIZE]),
        counter: ColorContainer::new([MaybePos::NONE; pos::BOARD_SIZE], [MaybePos::NONE; pos::BOARD_SIZE]),
    };

    pub fn update_quiet(&mut self, history: &History, quiet_plied: QuietPlied, color: Color, best_move: Pos, depth: Depth) {
        let bonus = depth * depth * params::HT_QUIET_BONUS_MUL;

        for &pos in quiet_plied.iter() {
            let bonus = bonus * Self::is_equal_sign(pos, best_move);

            Self::update_gravity_score(&mut self.quiet[color][pos.idx_usize()], bonus);
        }

        if let Some(prev_move) = history.last_action().ok() {
            self.counter[color][prev_move.idx_usize()] = best_move.into();
        }
    }

    pub fn update_tactical(&mut self, three_plied: TacticalPlied, four_plied: TacticalPlied, color: Color, best_move: Pos, depth: Depth) {
        let bonus = depth * depth * params::HT_TACTICAL_BONUS_MUL;

        for &pos in three_plied.iter() {
            let bonus = bonus * Self::is_equal_sign(pos, best_move);

            Self::update_gravity_score(&mut self.three[color][pos.idx_usize()], bonus);
        }

        for &pos in four_plied.iter() {
            let bonus = bonus * Self::is_equal_sign(pos, best_move);

            Self::update_gravity_score(&mut self.four[color][pos.idx_usize()], bonus);
        }
    }

    pub fn increase_age(&mut self) {
        for score in self.quiet.iter_mut().flatten()
            .chain(self.three.iter_mut().flatten())
            .chain(self.four.iter_mut().flatten())
        {
            *score = (*score as f64 * params::HT_AGEING_MUL) as i16;
        }
    }

    fn update_gravity_score(score: &mut i16, bonus: i32) {
        let current = *score as i32;

        *score = (current + bonus - current * bonus.abs() / MAX_HISTORY_SCORE)
            .clamp(-MAX_HISTORY_SCORE, MAX_HISTORY_SCORE)
            as i16;
    }

    fn is_equal_sign(pos: Pos, best_move: Pos) -> i32 {
        if pos == best_move {
            1
        } else {
            -1
        }
    }

}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct HistoryTableData {
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "rusty_renju::utils::serde::serialize_array",
            deserialize_with = "rusty_renju::utils::serde::deserialize_array"
        ),
    )]
    quiet: [u8; 2 * pos::BOARD_SIZE * 2],
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "rusty_renju::utils::serde::serialize_array",
            deserialize_with = "rusty_renju::utils::serde::deserialize_array"
        ),
    )]
    three: [u8; 2 * pos::BOARD_SIZE * 2],
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "rusty_renju::utils::serde::serialize_array",
            deserialize_with = "rusty_renju::utils::serde::deserialize_array"
        ),
    )]
    four: [u8; 2 * pos::BOARD_SIZE * 2],
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "rusty_renju::utils::serde::serialize_array",
            deserialize_with = "rusty_renju::utils::serde::deserialize_array"
        ),
    )]
    counter: [u8; pos::BOARD_SIZE * 2],
}

#[cfg(feature = "serde")]
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

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for HistoryTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let data = HistoryTableData::deserialize(deserializer)?;

        unsafe { Ok(Self {
            quiet: std::mem::transmute(data.quiet),
            three: std::mem::transmute(data.three),
            four: std::mem::transmute(data.four),
            counter: std::mem::transmute(data.counter),
        }) }
    }
}
