use crate::impl_debug_from_display;
use crate::notation::direction::Direction;
use crate::utils::str_utils::u8_from_str;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub const BOARD_WIDTH: u8 = 15;
pub const BOARD_SIZE: usize = U_BOARD_WIDTH * U_BOARD_WIDTH;
pub const BOARD_BOUND: usize = BOARD_SIZE - 1;

pub const U_BOARD_WIDTH: usize = BOARD_WIDTH as usize;
pub const I_BOARD_WIDTH: isize = BOARD_WIDTH as isize;

pub const U8_BOARD_SIZE: u8 = BOARD_SIZE as u8;
pub const U8_BOARD_BOUND: u8 = BOARD_BOUND as u8;

pub const CENTER: Pos = Pos::from_index(U8_BOARD_SIZE / 2);
pub const CENTER_ROW_COL: u8 = CENTER.col();

#[macro_export] macro_rules! cartesian_to_index {
    ($row:expr,$col:expr) => ($row * 15 + $col);
}

#[macro_export] macro_rules! index_to_cartesian {
    ($idx:expr) => (($idx / 15, $idx % 15));
}

#[macro_export] macro_rules! index_to_row {
    ($idx:expr) => ($idx / 15)
}

#[macro_export] macro_rules! index_to_col {
    ($idx:expr) => ($idx % 15);
}

#[macro_export] macro_rules! step_idx {
     ($direction:expr,$idx:expr,$amount:expr) => {
         match $direction {
            Direction::Horizontal => $idx + (1 * $amount),
            Direction::Vertical => $idx + (15 * $amount),
            Direction::Ascending => $idx + ((15 + 1) * $amount),
            Direction::Descending => $idx - ((15 - 1) * $amount)
        }
     };
}

#[macro_export] macro_rules! chebyshev_distance {
    ($ref_row:expr, $ref_col:expr, $row:expr, $col:expr) => {{
        let row_diff = ($ref_row - $row).abs();
        let col_diff = ($ref_col - $col).abs();

        row_diff.max(col_diff)
    }};
}

const STEP_TABLE: [isize; 4] = [1, I_BOARD_WIDTH, I_BOARD_WIDTH + 1, -(I_BOARD_WIDTH - 1)];

pub const fn pos_unchecked(source: &str) -> Pos {
    let row = u8_from_str(source, 1) - 1;
    let col = source.as_bytes()[0] - b'a';

    Pos::from_cartesian(row, col)
}

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct Pos(u8);

impl From<usize> for Pos {
    fn from(value: usize) -> Self {
        Pos::from_index(value as u8)
    }
}

impl From<u8> for Pos {
    fn from(value: u8) -> Self {
        Pos::from_index(value)
    }
}

impl Pos {

    pub const fn from_index(index: u8) -> Self {
        Self(index)
    }

    pub const fn from_cartesian(row: u8, col: u8) -> Self {
        Self(cartesian_to_index!(row, col))
    }


    pub const fn to_cartesian(&self) -> (u8, u8) {
        index_to_cartesian!(self.0)
    }

    pub const fn idx(&self) -> u8 {
        self.0
    }

    pub const fn idx_usize(&self) -> usize {
        self.0 as usize
    }

    pub const fn row(&self) -> u8 {
        index_to_row!(self.0)
    }

    pub const fn row_usize(&self) -> usize {
        self.row() as usize
    }

    pub const fn col(&self) -> u8 {
        index_to_col!(self.0)
    }

    pub const fn col_usize(&self) -> usize {
        self.col() as usize
    }

    pub const fn offset(&self, offset_row: isize, offset_col: isize) -> Option<Self> {
        let row = self.row() as isize + offset_row;
        let col = self.col() as isize + offset_col;

        if row >= 0 && row < I_BOARD_WIDTH && col >= 0 && col < I_BOARD_WIDTH {
            Some(Self::from_cartesian(row as u8, col as u8))
        } else {
            None
        }
    }

    pub const fn offset_unchecked(&self, offset_row: isize, offset_col: isize) -> Self {
        Self::from_cartesian(
            self.row_usize().saturating_add_signed(offset_row) as u8,
            self.col_usize().saturating_add_signed(offset_col) as u8
        )
    }

    pub const fn directional_offset_unchecked(&self, direction: Direction, offset: isize) -> Self {
        Self::from_index((self.0 as isize + (STEP_TABLE[direction as usize] * offset)) as u8)
    }

    pub fn distance(&self, other: Self) -> u8 {
        chebyshev_distance!(self.row() as i16, self.col() as i16, other.row() as i16, other.col() as i16) as u8
    }

}

#[derive(Debug)]
pub enum PosError {
    InvalidRowCharter,
    ColumnOrRowOutOfRange,
}

impl Display for PosError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRowCharter => write!(f, "Invalid row charter"),
            Self::ColumnOrRowOutOfRange => write!(f, "Column or row out of range"),
        }
    }
}

impl std::error::Error for PosError {}

impl FromStr for Pos {
    type Err = PosError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        source[1..].parse::<u8>()
            .map_err(|_| PosError::InvalidRowCharter)
            .and_then(|row| {
                let col = source.chars().next().unwrap() as u8 - b'a';

                (col < BOARD_WIDTH && row < BOARD_WIDTH)
                    .then(|| Pos::from_cartesian(row - 1 , col))
                    .ok_or(PosError::ColumnOrRowOutOfRange)
            })
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (self.col() + b'a') as char, self.row() + 1)
    }
}

impl_debug_from_display!(Pos);

impl Serialize for Pos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        if serializer.is_human_readable() {
            serializer.serialize_str(self.to_string().as_str())
        } else {
            serializer.serialize_u8(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for Pos {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        if deserializer.is_human_readable() {
            Self::from_str(&String::deserialize(deserializer)?).map_err(de::Error::custom)
        } else {
            Ok(Self::from_index(u8::deserialize(deserializer)?))
        }
    }
}

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
#[repr(transparent)]
pub struct MaybePos(Pos);

impl MaybePos {

    pub const INVALID_POS: Pos = Pos(u8::MAX);

    pub const NONE: Self = Self(Self::INVALID_POS);

    pub const fn new(pos: Pos) -> Self {
        Self(pos)
    }

    pub const fn is_none(&self) -> bool {
        self.0.0 == Self::INVALID_POS.0
    }

    pub const fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub const fn unwrap(self) -> Pos {
        assert!(self.is_some());
        self.0
    }

}

impl From<Pos> for MaybePos {
    fn from(value: Pos) -> Self {
        Self(value)
    }
}

impl From<MaybePos> for Option<Pos> {
    fn from(value: MaybePos) -> Self {
        if value.is_none() {
            None
        } else {
            Some(value.0)
        }
    }
}

impl From<Option<Pos>> for MaybePos {
    fn from(value: Option<Pos>) -> Self {
        match value {
            Some(pos) => Self(pos),
            None => Self::NONE,
        }
    }
}

impl FromStr for MaybePos {
    type Err = PosError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "none" => MaybePos::NONE,
            _ => s.parse::<Pos>()?.into()
        })
    }
}

impl Display for MaybePos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            MaybePos::NONE => write!(f, "none"),
            _ => write!(f, "{}", self.unwrap())
        }
    }
}

impl_debug_from_display!(MaybePos);

impl Serialize for MaybePos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        if serializer.is_human_readable() && self.is_none() {
            serializer.serialize_str("none")
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for MaybePos {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        if deserializer.is_human_readable() {
            Self::from_str(&String::deserialize(deserializer)?)
                .map_err(de::Error::custom)
        } else {
            Ok(Self(Pos::deserialize(deserializer)?))
        }
    }
}

