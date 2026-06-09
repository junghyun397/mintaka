use crate::board::Board;
use crate::board_io::{HISTORY_LITERAL_PASS, HISTORY_LITERAL_SEPARATOR};
use crate::impl_debug_from_display;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::{MaybePos, Pos};
use std::fmt::{Debug, Display, Formatter};
use std::iter;
use std::ops::{Index, IndexMut};
use std::str::FromStr;
#[cfg(feature = "typeshare")]
use typeshare::typeshare;
use crate::notation::rule::RuleKind;
use crate::utils::empty::Empty;

pub const MAX_HISTORY_SIZE: usize = 248;

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "Vec<MaybePos>"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "Vec<MaybePos>"))]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct History {
    pub entries: [MaybePos; MAX_HISTORY_SIZE],
    top: usize,
}

impl Empty for History {
    fn empty() -> Self {
        Self::EMPTY
    }
}

impl From<&[MaybePos]> for History {
    fn from(value: &[MaybePos]) -> Self {
        let mut history = History::empty();
        history.entries[.. value.len()].copy_from_slice(value);
        history.top = value.len();
        history
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
    pub const EMPTY: Self = Self {
        entries: [MaybePos::NONE; MAX_HISTORY_SIZE],
        top: 0,
    };

    pub fn len(&self) -> usize {
        self.top
    }

    pub fn actions(&self) -> &[MaybePos] {
        &self.entries[..self.top]
    }

    pub fn is_empty(&self) -> bool {
        self.top == 0
    }

    pub fn player_color(&self) -> Color {
        match self.top % 2 {
            0 => Color::Black,
            _ => Color::White,
        }
    }

    pub fn action(mut self, action: MaybePos) -> Self {
        self.action_mut(action);
        self
    }

    pub fn set(mut self, pos: Pos) -> Self {
        self.action_mut(pos.into());
        self
    }

    pub fn pass(mut self) -> Self {
        self.action_mut(MaybePos::NONE);
        self
    }

    pub fn pop(mut self) -> (Self, Option<MaybePos>) {
        let result = self.pop_mut();
        (self, result)
    }

    pub fn action_mut(&mut self, action: MaybePos) {
        assert!(self.top < pos::BOARD_SIZE);

        self.entries[self.top] = action;
        self.top += 1;
    }

    pub fn set_mut(&mut self, pos: Pos) {
        self.action_mut(pos.into())
    }

    pub fn pass_mut(&mut self) {
        self.action_mut(MaybePos::NONE)
    }

    pub fn pop_mut(&mut self) -> Option<MaybePos> {
        if self.top == 0 {
            return None;
        }

        self.top -= 1;
        Some(self.entries[self.top])
    }

    pub fn iter(&self) -> impl Iterator<Item = &MaybePos> {
        self.entries[..self.top].iter()
    }

    pub fn last_action_pair(&self) -> [MaybePos; 2] {
        match self.len() {
            0 => [MaybePos::NONE, MaybePos::NONE],
            1 => [MaybePos::NONE, self.entries[0]],
            _ => [self.entries[self.len() - 2], self.entries[self.len() - 1]]
        }
    }

    pub fn last_action(&self) -> Option<MaybePos> {
        if self.top == 0 {
            return None;
        }

        Some(self.entries[self.top - 1])
    }

    pub fn last_action_or_none(&self) -> MaybePos {
        if self.top == 0 {
            return MaybePos::NONE;
        }

        self.entries[self.top - 1]
    }

    pub fn last_action_unchecked(&self) -> MaybePos {
        self.entries[self.top - 1]
    }

    pub fn previous_action(&self) -> MaybePos {
        self.entries[self.top - 2]
    }

    pub fn avg_distance(&self, pos: Pos, sample_size: usize) -> u8 {
        if self.top == 0 {
            return 0;
        }

        let sample_size = self.top.min(sample_size);

        self.entries[self.top - sample_size .. self.top].iter()
            .map(|&action| action.distance_or(pos, 0) as u16)
            .sum::<u16>() as u8 / sample_size as u8
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
        let mut history = History::empty();
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

impl<const R: RuleKind> TryFrom<&Board<R>> for History {
    type Error = HistoryError;

    fn try_from(value: &Board<R>) -> Result<Self, Self::Error> {
        let mut black_history = vec![];
        let mut white_history = vec![];

        for pos in (1 .. pos::CENTER_ROW_COL).rev()
            .flat_map(|distance_from_center| {
                let begin_idx = pos::CENTER_ROW_COL - distance_from_center;
                let end_idx = pos::CENTER_ROW_COL + distance_from_center;

                (0 .. distance_from_center * 2 + 1) // horizontal-down
                    .map(move |offset| Pos::from_cartesian(begin_idx, begin_idx + offset))
                    .chain((0 .. distance_from_center * 2 + 1)
                        .map(move |offset| Pos::from_cartesian(end_idx, begin_idx + offset))
                    ) // horizontal-up
                    .chain((0 .. (distance_from_center * 2 + 1).saturating_sub(2))
                        .map(move |offset| Pos::from_cartesian(begin_idx + 1 + offset, begin_idx))
                    ) // vertical-left
                    .chain((0 .. (distance_from_center * 2 + 1).saturating_sub(2))
                        .map(move |offset| Pos::from_cartesian(begin_idx + 1 + offset, end_idx))
                    ) // vertical-right
            })
            .chain(iter::once(pos::CENTER))
        {
            match value.stone_kind(pos) {
                Some(Color::Black) => black_history.push(pos),
                Some(Color::White) => white_history.push(pos),
                _ => {},
            }
        }

        if white_history.len() + black_history.len() > pos::BOARD_SIZE {
            return Err(HistoryError::HistoryTooLong)
        }

        if white_history.len() > black_history.len() {
            return Err(HistoryError::WhiteIsLongerThanBlack);
        }

        let mut history = History::empty();

        while let Some(white_pos) = white_history.pop()
            && let Some(black_pos) = black_history.pop()
        {
            history.set_mut(black_pos);
            history.set_mut(white_pos);
        }

        if let Some(black_pos) = black_history.pop() {
            history.set_mut(black_pos);
        }

        Ok(history)
    }
}

impl From<&History> for Vec<MaybePos> {
    fn from(value: &History) -> Self {
        value.actions().to_vec()
    }
}

impl TryFrom<Vec<MaybePos>> for History {
    type Error = HistoryError;

    fn try_from(value: Vec<MaybePos>) -> Result<Self, Self::Error> {
        let top = value.len();

        if top > pos::BOARD_SIZE {
            return Err(HistoryError::HistoryTooLong);
        }

        let mut entries = [MaybePos::NONE; MAX_HISTORY_SIZE];
        entries[.. top].copy_from_slice(&value);

        Ok(Self {
            entries,
            top
        })
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for History {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.collect_seq(self.iter())
    }
}
