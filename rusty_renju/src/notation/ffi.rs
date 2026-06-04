use crate::board::Board;
use crate::notation::pos::{MaybePos, Pos};
use crate::notation::rule::RuleKind;

#[derive(Debug, Clone, Copy)]
pub enum AnyBoard {
    Renju(Board<{ RuleKind::Renju }>),
    Gomoku(Board<{ RuleKind::Gomoku }>),
    Freestyle(Board<{ RuleKind::Freestyle }>),
}

impl AnyBoard {
    pub fn rule_kind(&self) -> RuleKind {
        match self {
            Self::Renju(_) => RuleKind::Renju,
            Self::Gomoku(_) => RuleKind::Gomoku,
            Self::Freestyle(_) => RuleKind::Freestyle,
        }
    }
}

#[macro_export] macro_rules! dispatch_any_board {
    ($board:expr,$inner:ident => $body:expr) => {
        match $board {
            rusty_renju::notation::ffi::AnyBoard::Renju($inner) => $body,
            rusty_renju::notation::ffi::AnyBoard::Gomoku($inner) => $body,
            rusty_renju::notation::ffi::AnyBoard::Freestyle($inner) => $body,
        }
    };
    (wrap $board:expr,$inner:ident => $body:expr) => {
        match $board {
            rusty_renju::notation::ffi::AnyBoard::Renju($inner) => rusty_renju::notation::ffi::AnyBoard::Renju($body),
            rusty_renju::notation::ffi::AnyBoard::Gomoku($inner) => rusty_renju::notation::ffi::AnyBoard::Gomoku($body),
            rusty_renju::notation::ffi::AnyBoard::Freestyle($inner) => rusty_renju::notation::ffi::AnyBoard::Freestyle($body),
        }
    };
    (wrap $rule_kind:expr,$body:expr) => {
        match $rule_kind {
            rusty_renju::notation::rule::RuleKind::Renju => rusty_renju::notation::ffi::AnyBoard::Renju($body),
            rusty_renju::notation::rule::RuleKind::Gomoku => rusty_renju::notation::ffi::AnyBoard::Gomoku($body),
            rusty_renju::notation::rule::RuleKind::Freestyle => rusty_renju::notation::ffi::AnyBoard::Freestyle($body),
        }
    }
}

pub fn from_raw_maybe_pos_slice<'a>(slice: *const u8, len: usize) -> Option<&'a [MaybePos]> {
    if len == 0 {
        return Some(&[]);
    }

    if slice.is_null() {
        return None;
    }

    Some(unsafe { std::slice::from_raw_parts(slice as *const MaybePos, len) })
}

pub fn from_raw_pos_slice<'a>(slice: *const u8, len: usize) -> Option<&'a [Pos]> {
    if len == 0 {
        return Some(&[]);
    }

    if slice.is_null() {
        return None;
    }

    Some(unsafe { std::slice::from_raw_parts(slice as *const Pos, len) })
}
