use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

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
pub extern "C" fn rusty_renju_forbidden_kind_none() -> u8 { 0 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_forbidden_kind_double_three() -> u8 { rusty_renju::notation::rule::ForbiddenKind::DoubleThree as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_forbidden_kind_double_four() -> u8 { rusty_renju::notation::rule::ForbiddenKind::DoubleFour as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_forbidden_kind_double_overline() -> u8 { rusty_renju::notation::rule::ForbiddenKind::Overline as u8 }

pub const MAYBE_POS_NONE: u8 = rusty_renju::notation::pos::MaybePos::INVALID_POS.idx();

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_pos_none() -> u8 { MAYBE_POS_NONE }

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BoardExportStone {
    pub color: u8,
    pub sequence: u8,
}

impl BoardExportStone {
    const EMPTY: Self = Self { color: COLOR_NONE, sequence: 0 };
}

const BOARD_EXPORT_ITEM_EMPTY: u8 = 0;
const BOARD_EXPORT_ITEM_STONE: u8 = 1;
const BOARD_EXPORT_ITEM_FORBIDDEN: u8 = 2;

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_export_item_empty() -> u8 { BOARD_EXPORT_ITEM_EMPTY }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_export_item_stone() -> u8 { BOARD_EXPORT_ITEM_STONE }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_export_item_forbidden() -> u8 { BOARD_EXPORT_ITEM_FORBIDDEN }

type Patterns = rusty_renju::notation::color::ColorContainer<[rusty_renju::pattern::Pattern; rusty_renju::pattern::PATTERN_SIZE]>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BoardExportItem {
    pub kind: u8,
    pub stone: BoardExportStone,
    pub forbidden_kind: u8,

}

impl From<rusty_renju::board_iter::BoardExportItem> for BoardExportItem {
    fn from(value: rusty_renju::board_iter::BoardExportItem) -> Self {
        match value {
            rusty_renju::board_iter::BoardExportItem::Empty => BoardExportItem {
                kind: BOARD_EXPORT_ITEM_EMPTY,
                stone: BoardExportStone::EMPTY,
                forbidden_kind: 0,
            },
            rusty_renju::board_iter::BoardExportItem::Stone(stone) => BoardExportItem {
                kind: BOARD_EXPORT_ITEM_STONE,
                stone: BoardExportStone { color: stone.color as u8, sequence: stone.sequence },
                forbidden_kind: 0
            },
            rusty_renju::board_iter::BoardExportItem::Forbidden(kind) => BoardExportItem {
                kind: BOARD_EXPORT_ITEM_FORBIDDEN,
                stone: BoardExportStone::EMPTY,
                forbidden_kind: kind as u8,
            },
        }
    }
}

#[repr(C)]
pub struct BoardDescribe {
    pub hash_key: u64,
    pub player_color: u8,
    pub field: [BoardExportItem; rusty_renju::notation::pos::BOARD_SIZE],
}

impl From<rusty_renju::board_io::BoardDescribe> for BoardDescribe {
    fn from(value: rusty_renju::board_io::BoardDescribe) -> Self {
        let hash_key = u64::from(value.hash_key);
        let player_color = value.player_color as u8;
        let field = std::array::from_fn(|idx| {
            value.field.get(idx)
                .copied()
                .map(BoardExportItem::from)
                .unwrap_or_else(|| rusty_renju::board_iter::BoardExportItem::Empty.into())
        });

        Self {
            hash_key,
            player_color,
            field,
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_default_board() -> *mut rusty_renju::notation::ffi::CBoard {
    Box::into_raw(Box::new(rusty_renju::board::Board::default().into()))
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
pub extern "C" fn rusty_renju_board_hash_key(board: *const rusty_renju::notation::ffi::CBoard) -> u64 {
    unsafe { board.as_ref() }
        .map_or(0, |board| u64::from(board.inner.hash_key))
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_player_color(board: *const rusty_renju::notation::ffi::CBoard) -> u8 {
    unsafe { board.as_ref() }
        .map_or(0, |board| board.inner.player_color as u8)
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_stones(board: *const rusty_renju::notation::ffi::CBoard) -> u8 {
    unsafe { board.as_ref() }
        .map_or(0, |board| board.inner.stones)
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_patterns(board: *const rusty_renju::notation::ffi::CBoard) -> *mut Patterns {
    if let Some(board) = unsafe { board.as_ref() } {
        Box::into_raw(Box::new(board.inner.patterns.field.into()))
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_patterns_free(patterns: *mut Patterns) {
    if !patterns.is_null() {
        unsafe { drop(Box::from_raw(patterns)); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_pattern(board: *const rusty_renju::notation::ffi::CBoard, color: u8, pos: u8) -> u32 {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(color) = rusty_renju::notation::color::Color::try_from(color)
        && let Ok(pos) = rusty_renju::notation::pos::Pos::try_from(pos)
    {
        board.inner.patterns.field[color][pos.idx_usize()].into()
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_is_pos_empty(board: *const rusty_renju::notation::ffi::CBoard, pos: u8) -> bool {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(pos) = rusty_renju::notation::pos::Pos::try_from(pos)
    {
        board.inner.is_pos_empty(pos)
    } else {
        false
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_is_legal_move(board: *const rusty_renju::notation::ffi::CBoard, pos: u8) -> bool {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(pos) = rusty_renju::notation::pos::Pos::try_from(pos)
    {
        board.inner.is_legal_move(pos)
    } else {
        false
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_stone_kind(board: *const rusty_renju::notation::ffi::CBoard, pos: u8) -> u8 {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(pos) = rusty_renju::notation::pos::Pos::try_from(pos)
    {
        board.inner.stone_kind(pos).map_or(COLOR_NONE, |color| color as u8)
    } else {
        COLOR_NONE
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
pub extern "C" fn rusty_renju_board_describe(board: *const rusty_renju::notation::ffi::CBoard, maybe_pos_slice: *const u8, len: usize) -> *mut BoardDescribe {
    if let Some(board) = unsafe { board.as_ref() }
        && let Some(maybe_pos_slice) = rusty_renju::notation::ffi::from_raw_maybe_pos_slice(maybe_pos_slice, len)
    {
        Box::into_raw(Box::new(BoardDescribe::from(board.inner.describe(&maybe_pos_slice.into()))))
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_describe_free(describe: *mut BoardDescribe) {
    if !describe.is_null() {
        unsafe { drop(Box::from_raw(describe)); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_version() -> *const c_char {
    CString::new(env!("CARGO_PKG_VERSION")).unwrap().into_raw()
}
