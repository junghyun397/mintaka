use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use rusty_renju::utils::empty::Empty;

pub const COLOR_NONE: u8 = u8::MAX;

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_color_black() -> u8 { rusty_renju::notation::color::Color::Black as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_color_white() -> u8 { rusty_renju::notation::color::Color::White as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_color_none() -> u8 { COLOR_NONE }

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_rule_gomoku() -> u8 { rusty_renju::notation::rule::RuleKind::Gomoku as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_rule_renju() -> u8 { rusty_renju::notation::rule::RuleKind::Renju as u8 }

const FORBIDDEN_KIND_NONE: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_forbidden_kind_none() -> u8 { FORBIDDEN_KIND_NONE }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_forbidden_kind_double_three() -> u8 { rusty_renju::notation::rule::ForbiddenKind::DoubleThree as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_forbidden_kind_double_four() -> u8 { rusty_renju::notation::rule::ForbiddenKind::DoubleFour as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_forbidden_kind_overline() -> u8 { rusty_renju::notation::rule::ForbiddenKind::Overline as u8 }

pub const MAYBE_POS_NONE: u8 = rusty_renju::notation::pos::MaybePos::INVALID_POS.idx();

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_pos_none() -> u8 { MAYBE_POS_NONE }

const BOARD_EXPORT_ITEM_EMPTY: u8 = 0;
const BOARD_EXPORT_ITEM_STONE: u8 = 1;
const BOARD_EXPORT_ITEM_FORBIDDEN: u8 = 2;

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_export_item_empty() -> u8 { BOARD_EXPORT_ITEM_EMPTY }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_export_item_stone() -> u8 { BOARD_EXPORT_ITEM_STONE }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_export_item_forbidden() -> u8 { BOARD_EXPORT_ITEM_FORBIDDEN }

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_closed_four_mask() -> u32 { rusty_renju::pattern::UNIT_CLOSED_FOUR_MASK }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_open_four_mask() -> u32 { rusty_renju::pattern::UNIT_OPEN_FOUR_MASK }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_open_three_mask() -> u32 { rusty_renju::pattern::UNIT_OPEN_THREE_MASK }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_close_three_mask() -> u32 { rusty_renju::pattern::UNIT_CLOSE_THREE_MASK }

#[repr(C)]
pub struct BoardExportItem {
    pub kind: u8,
    pub stone: u8,
    pub forbidden_kind: u8,
}

impl From<rusty_renju::board_iter::BoardExportItem> for BoardExportItem {
    fn from(value: rusty_renju::board_iter::BoardExportItem) -> Self {
        match value {
            rusty_renju::board_iter::BoardExportItem::Empty => BoardExportItem {
                kind: BOARD_EXPORT_ITEM_EMPTY,
                stone: COLOR_NONE,
                forbidden_kind: 0,
            },
            rusty_renju::board_iter::BoardExportItem::Stone(stone) => BoardExportItem {
                kind: BOARD_EXPORT_ITEM_STONE,
                stone: stone as u8,
                forbidden_kind: 0
            },
            rusty_renju::board_iter::BoardExportItem::Forbidden(kind) => BoardExportItem {
                kind: BOARD_EXPORT_ITEM_FORBIDDEN,
                stone: COLOR_NONE,
                forbidden_kind: kind as u8,
            },
        }
    }
}

#[repr(C)]
pub struct BoardWinner {
    pub is_some: u8,
    pub color: u8,
    pub sequence: [u8; 5],
}

impl From<Option<rusty_renju::board_utils::BoardWinner>> for BoardWinner {
    fn from(value: Option<rusty_renju::board_utils::BoardWinner>) -> Self {
        BoardWinner {
            is_some: value.is_some() as u8,
            color: value.as_ref().map_or(COLOR_NONE, |winner|
                winner.color as u8
            ),
            sequence: value.as_ref().map_or([MAYBE_POS_NONE; 5], |winner|
                winner.moves.map(|pos| pos.idx())
            ),
        }
    }
}

#[repr(C)]
pub struct BoardDescribe {
    pub hash_key: u64,
    pub player_color: u8,
    pub field: [BoardExportItem; rusty_renju::notation::pos::BOARD_SIZE],
    pub winner: BoardWinner
}

impl From<rusty_renju::board_io::BoardDescribe> for BoardDescribe {
    fn from(value: rusty_renju::board_io::BoardDescribe) -> Self {
        Self {
            hash_key: u64::from(value.hash_key),
            player_color: value.player_color as u8,
            field: std::array::from_fn(|idx| {
                value.field.get(idx)
                    .copied()
                    .map(BoardExportItem::from)
                    .unwrap_or_else(|| rusty_renju::board_iter::BoardExportItem::Empty.into())
            }),
            winner: value.winner.into()
        }
    }
}

#[repr(C)]
pub struct BoardPattens {
    pub black_pattens: [u32; rusty_renju::notation::pos::BOARD_SIZE],
    pub white_pattens: [u32; rusty_renju::notation::pos::BOARD_SIZE],
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_default_board() -> *mut rusty_renju::notation::ffi::CBoard {
    Box::into_raw(Box::new(rusty_renju::board::Board::empty().into()))
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_empty_hash() -> u64 {
    u64::from(rusty_renju::memo::hash_key::HashKey::EMPTY)
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_from_history(actions: *const u8, len: usize) -> *mut rusty_renju::notation::ffi::CBoard {
    if let Some(actions) = rusty_renju::notation::ffi::from_raw_maybe_pos_slice(actions, len) {
        Box::into_raw(Box::new(rusty_renju::board::Board::from(&actions.into()).into()))
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_from_string(source: *const c_char) -> *mut rusty_renju::notation::ffi::CBoard {
    if let Some(source) = unsafe { source.as_ref() }
        && let Ok(source) = unsafe { CStr::from_ptr(source) }.to_str()
        && let Ok(board) = source.parse::<rusty_renju::board::Board>()
    {
        Box::into_raw(Box::new(board.into()))
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_to_string(board: *const rusty_renju::notation::ffi::CBoard) -> *mut c_char {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(result) = CString::new(board.inner.to_string())
    {
        result.into_raw()
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_set(board: *const rusty_renju::notation::ffi::CBoard, pos: u8) -> *mut rusty_renju::notation::ffi::CBoard {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(action) = rusty_renju::notation::pos::MaybePos::try_from(pos)
    {
        let board = match action {
            rusty_renju::notation::pos::MaybePos::NONE => board.inner.pass(),
            pos => board.inner.set(pos.unwrap()),
        }.into();

        Box::into_raw(Box::new(board))
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_unset(board: *const rusty_renju::notation::ffi::CBoard, pos: u8) -> *mut rusty_renju::notation::ffi::CBoard {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(maybe_pos) = rusty_renju::notation::pos::MaybePos::try_from(pos)
    {
        let board = match maybe_pos {
            rusty_renju::notation::pos::MaybePos::NONE => board.inner.pass(),
            pos => board.inner.unset(pos.unwrap()),
        }.into();

        Box::into_raw(Box::new(board))
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_free(board: *mut rusty_renju::notation::ffi::CBoard) {
    if !board.is_null() {
        unsafe { drop(Box::from_raw(board)); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_describe_into(
    board: *const rusty_renju::notation::ffi::CBoard,
    out: *mut BoardDescribe,
) -> bool {
    let Some(board) = (unsafe { board.as_ref() }) else {
        return false;
    };

    unsafe { out.write(BoardDescribe::from(board.inner.describe())) }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_pattens_into(
    board: *const rusty_renju::notation::ffi::CBoard,
    out: *mut BoardPattens,
) -> bool {
    let Some(board) = (unsafe { board.as_ref() }) else {
        return false;
    };

    unsafe { out.write(BoardPattens {
        black_pattens: board.inner.patterns.field[rusty_renju::notation::color::Color::Black]
            .map(u32::from)[..rusty_renju::notation::pos::BOARD_SIZE]
            .try_into().unwrap(),
        white_pattens: board.inner.patterns.field[rusty_renju::notation::color::Color::White]
            .map(u32::from)[..rusty_renju::notation::pos::BOARD_SIZE]
            .try_into().unwrap(),

    })}
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_version() -> *const c_char {
    CString::new(env!("CARGO_PKG_VERSION")).unwrap().into_raw()
}
