use crate::value;
use crate::value::Depth;
use core::f64;
use rusty_renju::notation::color::{Color, ColorContainer};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use serde::{Deserialize, Serialize, Serializer};

#[derive(Copy, Clone)]
pub struct HistoryTable {
    pub quiet: ColorContainer<[i16; pos::BOARD_SIZE]>,
    pub tactical: ColorContainer<[i16; pos::BOARD_SIZE]>,
    pub counter: ColorContainer<[[MaybePos; pos::BOARD_SIZE]; pos::BOARD_SIZE]>,
}

impl HistoryTable {

    pub fn new() -> Self {
        Self {
            quiet: ColorContainer::new([0; pos::BOARD_SIZE], [0; pos::BOARD_SIZE]),
            tactical: ColorContainer::new([0; pos::BOARD_SIZE], [0; pos::BOARD_SIZE]),
            counter: ColorContainer::new([[MaybePos::NONE; pos::BOARD_SIZE]; pos::BOARD_SIZE], [[MaybePos::NONE; pos::BOARD_SIZE]; pos::BOARD_SIZE]),
        }
    }

    pub fn update_fail_high(&self, color: Color, pos: Pos, depth: Depth) {
    }

    pub fn update_fail_low(&self, color: Color, pos: Pos, depth: Depth) {
    }

    pub fn increase_age(&mut self) {
        fn ageing_score(score: &mut i16) {
            *score = (*score as f64 * value::HISTORY_TABLE_AGEING_MUL) as i16;
        }

        self.quiet[Color::Black].iter_mut().for_each(ageing_score);
        self.quiet[Color::White].iter_mut().for_each(ageing_score);

        self.tactical[Color::Black].iter_mut().for_each(ageing_score);
        self.tactical[Color::White].iter_mut().for_each(ageing_score);
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
    attack: [u8; 2 * pos::BOARD_SIZE * 2],
    #[serde(
        serialize_with = "crate::utils::serde::serialize_array",
        deserialize_with = "crate::utils::serde::deserialize_array"
    )]
    counter: [u8; pos::BOARD_SIZE * pos::BOARD_SIZE * 2],
}

impl Serialize for HistoryTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let data = unsafe { HistoryTableData {
            quiet: std::mem::transmute(self.quiet),
            attack: std::mem::transmute(self.tactical),
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
            tactical: std::mem::transmute(data.quiet),
            counter: std::mem::transmute(data.counter),
        }) }
    }
}
