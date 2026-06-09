use crate::params;
use crate::value::Depth;
use core::f64;
use rusty_renju::history::History;
use rusty_renju::notation::color::{Color, ColorContainer};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos, PosList};
use rusty_renju::utils::empty::Empty;

pub type QuietPlied = PosList<{ 256 - 8 }>;
pub type TacticalPlied = PosList<{ 64 - 8 }>;

pub const MAX_HISTORY_SCORE: i32 = (i16::MAX / 2) as i32;

#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HistoryTable {
    #[cfg_attr(feature = "serde", serde(serialize_with = "rusty_renju::utils::serde::serialize_color_container_array", deserialize_with = "rusty_renju::utils::serde::deserialize_color_container_array"), )]
    pub quiet: ColorContainer<[i16; pos::BOARD_SIZE]>,
    #[cfg_attr(feature = "serde", serde(serialize_with = "rusty_renju::utils::serde::serialize_color_container_array", deserialize_with = "rusty_renju::utils::serde::deserialize_color_container_array"), )]
    pub three: ColorContainer<[i16; pos::BOARD_SIZE]>,
    #[cfg_attr(feature = "serde", serde(serialize_with = "rusty_renju::utils::serde::serialize_color_container_array", deserialize_with = "rusty_renju::utils::serde::deserialize_color_container_array"), )]
    pub four: ColorContainer<[i16; pos::BOARD_SIZE]>,
    #[cfg_attr(feature = "serde", serde(serialize_with = "rusty_renju::utils::serde::serialize_color_container_array", deserialize_with = "rusty_renju::utils::serde::deserialize_color_container_array"), )]
    pub counter: ColorContainer<[MaybePos; pos::BOARD_SIZE]>,
}

impl Empty for HistoryTable {
    fn empty() -> Self {
        Self {
            quiet: ColorContainer::new([0; pos::BOARD_SIZE], [0; pos::BOARD_SIZE]),
            three: ColorContainer::new([0; pos::BOARD_SIZE], [0; pos::BOARD_SIZE]),
            four: ColorContainer::new([0; pos::BOARD_SIZE], [0; pos::BOARD_SIZE]),
            counter: ColorContainer::new([MaybePos::NONE; pos::BOARD_SIZE], [MaybePos::NONE; pos::BOARD_SIZE]),
        }
    }
}

impl HistoryTable {
    pub fn update_quiet(&mut self, history: &History, quiet_plied: QuietPlied, color: Color, best_move: Pos, depth: Depth) {
        let bonus = depth * depth * params::HT_QUIET_BONUS_MUL;

        for &pos in quiet_plied.iter() {
            let bonus = bonus * Self::is_equal_sign(pos, best_move);

            Self::update_gravity_score(&mut self.quiet[color][pos.idx_usize()], bonus);
        }

        if history.len() > 0
            && let Some(prev_move) = history.last_action_unchecked().ok() 
        {
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
        for score in self.quiet.0.iter_mut().flatten()
            .chain(self.three.0.iter_mut().flatten())
            .chain(self.four.0.iter_mut().flatten())
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
