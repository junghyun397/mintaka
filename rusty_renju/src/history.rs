use crate::board::Board;
use crate::board_io::{HISTORY_LITERAL_PASS, HISTORY_LITERAL_SEPARATOR};
use crate::impl_debug_from_display;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::{MaybePos, Pos};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Index, IndexMut};
use std::str::FromStr;

#[derive(Copy, Clone)]
pub struct History {
    pub entries: [MaybePos; pos::BOARD_SIZE],
    top: usize
}

impl Default for History {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl Index<usize> for History {
    type Output = MaybePos;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for History {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl History {

    const EMPTY: Self = Self {
        entries: [MaybePos::NONE; pos::BOARD_SIZE],
        top: 0,
    };

    pub fn len(&self) -> usize {
        self.top
    }

    pub fn slice(&self) -> &[MaybePos] {
        &self.entries[..self.top]
    }

    pub fn pop_mut(&mut self) -> Option<MaybePos> {
        if self.top == 0 {
            return None;
        }

        self.top -= 1;
        Some(self.entries[self.top])
    }

    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

    pub fn action_mut(&mut self, action: MaybePos) {
        self.entries[self.top] = action;
        self.top += 1;
    }

    pub fn set_mut(&mut self, pos: Pos) {
        self.action_mut(pos.into())
    }

    pub fn pass_mut(&mut self) {
        self.action_mut(MaybePos::NONE)
    }

    pub fn iter(&self) -> impl Iterator<Item = &MaybePos> {
        self.entries[..self.top].iter()
    }

    pub fn recent_move_pair(&self) -> [Option<MaybePos>; 2] {
        match self.len() {
            0 => [None, None],
            1 => [Some(self.entries[1]), None],
            _ => [Some(self.entries[self.len() - 2]), Some(self.entries[self.len() - 1])]
        }
    }

    pub fn recent_move_unchecked(&self) -> Pos {
        debug_assert_ne!(self.top, 0);
        self.entries[self.top].unwrap()
    }

    pub fn recent_opponent_move_unchecked(&self) -> Pos {
        debug_assert!(self.top > 0);
        self.entries[self.top - 1].unwrap()
    }

    pub fn recent_player_move_unchecked(&self) -> Pos {
        debug_assert!(self.top > 1);
        self.entries[self.top - 2].unwrap()
    }

    pub fn recent_move_pair_unchecked(&self) -> [Pos; 2] {
        debug_assert!(self.top > 0);
        [self.recent_player_move_unchecked(), self.recent_opponent_move_unchecked()]
    }

    pub fn avg_distance_to_recent_moves(&self, pos: Pos) -> u8 {
        if self.top > 3 {
            let distance1 = self.entries[self.top - 4].unwrap().distance(pos);
            let distance2 = self.entries[self.top - 3].unwrap().distance(pos);
            let distance3 = self.entries[self.top - 2].unwrap().distance(pos);
            let distance4 = self.entries[self.top - 1].unwrap().distance(pos);
            return (distance1 + distance2 + distance3 + distance4) / 4
        }

        match self.top {
            1 => self.entries[0].unwrap().distance(pos),
            2 => {
                let distance1 = self.entries[self.top - 2].unwrap().distance(pos);
                let distance2 = self.entries[self.top - 1].unwrap().distance(pos);
                (distance1 + distance2) / 2
            },
            3 => {
                let distance1 = self.entries[self.top - 3].unwrap().distance(pos);
                let distance2 = self.entries[self.top - 2].unwrap().distance(pos);
                let distance3 = self.entries[self.top - 1].unwrap().distance(pos);
                (distance1 + distance2 + distance3) / 3
            },
            _ => 0
        }
    }

}

impl Display for History {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let history = self.iter()
            .map(|&action|
                match action {
                    MaybePos::NONE => HISTORY_LITERAL_PASS.to_string(),
                    pos => pos.unwrap().to_string()
                }
            )
            .collect::<Vec<_>>()
            .join(HISTORY_LITERAL_SEPARATOR);

        write!(f, "{history}")
    }
}

impl_debug_from_display!(History);

impl FromStr for History {
    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let mut history = History::default();
        let source = source.to_lowercase();
        let bytes = source.as_bytes();
        let mut idx = 0;

        fn detect_token(bytes: &[u8]) -> Option<(Pos, usize)> {
            if bytes.len() < 2
                || !(b'a' .. (b'a' + pos::BOARD_WIDTH)).contains(&bytes[0])
            {
                return None;
            }

            let len =
                if bytes.len() > 2 && bytes[2].is_ascii_digit() {
                    3
                } else if bytes[1].is_ascii_digit() {
                    2
                } else {
                    return None;
                };

            Pos::from_str(str::from_utf8(&bytes[.. len]).unwrap()).ok()
                .map(|pos| (pos, len))
        }

        while idx < bytes.len() {
            if let Some((pos, len)) = detect_token(&bytes[idx ..]) {
                history.set_mut(pos);
                idx += len;
            } else {
                idx += 1;
            }
        }

        Ok(history)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum HistoryError {
    HistoryTooLong,
    WhiteIsLongerThanBlack,
}

impl Display for HistoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryError::HistoryTooLong => write!(f, "history is too long"),
            HistoryError::WhiteIsLongerThanBlack => write!(f, "white's history is longer than black's history"),
        }
    }
}

impl std::error::Error for HistoryError {}

impl TryFrom<&Board> for History {
    type Error = HistoryError;

    fn try_from(value: &Board) -> Result<Self, Self::Error> {
        let mut black_history = vec![];
        let mut white_history = vec![];

        for distance_from_center in 0 .. pos::CENTER_ROW_COL {
            let begin_idx = pos::CENTER_ROW_COL - distance_from_center;
            let end_idx = pos::CENTER_ROW_COL + distance_from_center;

            for pos in
                (0 .. distance_from_center * 2 + 1) // horizontal-up
                    .map(|offset| Pos::from_cartesian(begin_idx, begin_idx + offset))
                    .chain((0 .. distance_from_center * 2 + 1)
                        .map(|offset| Pos::from_cartesian(end_idx, begin_idx + offset))
                    ) // horizontal-down
                    .chain((0 .. (distance_from_center * 2 + 1).saturating_sub(2))
                        .map(|offset| Pos::from_cartesian(begin_idx + 1 + offset, begin_idx))
                    ) // vertical-left
                    .chain((0 .. (distance_from_center * 2 + 1).saturating_sub(2))
                        .map(|offset| Pos::from_cartesian(begin_idx + 1 + offset, end_idx))
                    ) // vertical-right
            {
                match value.stone_kind(pos) {
                    Some(Color::Black) => black_history.push(pos),
                    Some(Color::White) => white_history.push(pos),
                    _ => {}
                }
            }
        }

        if white_history.len() + black_history.len() > pos::BOARD_SIZE {
            return Err(HistoryError::HistoryTooLong)
        }

        if white_history.len() > black_history.len() {
            return Err(HistoryError::WhiteIsLongerThanBlack);
        }

        let mut history = History::default();

        while let Some(black_pos) = black_history.pop() && let Some(white_pos) = white_history.pop() {
            history.set_mut(black_pos);
            history.set_mut(white_pos);
        }

        if let Some(black_pos) = black_history.pop() {
            history.set_mut(black_pos);
        }

        Ok(history)
    }
}

impl Serialize for History {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.collect_seq(self.iter())
    }
}

impl<'de> Deserialize<'de> for History {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let vector = Vec::<MaybePos>::deserialize(deserializer)?;
        let top = vector.len();

        if top > pos::BOARD_SIZE {
            return Err(serde::de::Error::custom("history is too long"));
        }

        let mut entries = [MaybePos::NONE; pos::BOARD_SIZE];
        entries[.. top].copy_from_slice(&vector);

        Ok(Self {
            entries,
            top
        })
    }
}
