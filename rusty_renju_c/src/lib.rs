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

pub const MAYBE_POS_NONE: u8 = rusty_renju::notation::pos::MaybePos::INVALID_POS.idx();

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_pos_none() -> u8 { MAYBE_POS_NONE }

#[repr(C)]
pub struct Board {
    inner: rusty_renju::board::Board,
}

impl From<rusty_renju::board::Board> for Board {
    fn from(value: rusty_renju::board::Board) -> Self {
        Self { inner: value }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BoardExportStone {
    pub color: u8,
    pub sequence: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union BoardExportItemData {
    pub stone: BoardExportStone,
    pub forbidden_kind: u8,
}

const BOARD_EXPORT_ITEM_EMPTY: u8 = 0;
const BOARD_EXPORT_ITEM_STONE: u8 = 1;
const BOARD_EXPORT_ITEM_FORBIDDEN: u8 = 2;

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_export_item_empty() -> u8 { BOARD_EXPORT_ITEM_EMPTY }
#[no_mangle]
pub extern "C" fn rusty_renju_board_export_item_stone() -> u8 { BOARD_EXPORT_ITEM_STONE }
#[no_mangle]
pub extern "C" fn rusty_renju_board_export_item_forbidden() -> u8 { BOARD_EXPORT_ITEM_FORBIDDEN }

type Patterns = rusty_renju::notation::color::ColorContainer<[rusty_renju::pattern::Pattern; rusty_renju::pattern::PATTERN_SIZE]>;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BoardExportItem {
    pub kind: u8,
    pub data: BoardExportItemData,
}

impl From<rusty_renju::board_iter::BoardExportItem> for BoardExportItem {
    fn from(value: rusty_renju::board_iter::BoardExportItem) -> Self {
        match value {
            rusty_renju::board_iter::BoardExportItem::Empty => BoardExportItem {
                kind: BOARD_EXPORT_ITEM_EMPTY,
                data: BoardExportItemData { forbidden_kind: 0 },
            },
            rusty_renju::board_iter::BoardExportItem::Stone(stone) => BoardExportItem {
                kind: BOARD_EXPORT_ITEM_STONE,
                data: BoardExportItemData {
                    stone: BoardExportStone { color: stone.color as u8, sequence: stone.sequence },
                },
            },
            rusty_renju::board_iter::BoardExportItem::Forbidden(kind) => BoardExportItem {
                kind: BOARD_EXPORT_ITEM_FORBIDDEN,
                data: BoardExportItemData { forbidden_kind: kind as u8 },
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

fn actions_slice<'a>(actions: *const u8, len: usize) -> Option<&'a [u8]> {
    if len == 0 {
        return Some(&[]);
    }

    if actions.is_null() {
        return None;
    }

    Some(unsafe { std::slice::from_raw_parts(actions, len) })
}

fn history_from_actions(actions: &[u8]) -> Option<rusty_renju::history::History> {
    let mut history = rusty_renju::history::History::default();

    for &action in actions {
        let action = rusty_renju::notation::pos::MaybePos::try_from(action).ok()?;
        history.action_mut(action);
    }

    Some(history)
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_default_board() -> *mut Board {
    Box::into_raw(Box::new(rusty_renju::board::Board::default().into()))
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_empty_hash() -> u64 {
    u64::from(rusty_renju::memo::hash_key::HashKey::EMPTY)
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_from_history(actions: *const u8, len: usize) -> *mut Board {
    if let Some(actions) = actions_slice(actions, len)
        && let Some(history) = history_from_actions(actions)
    {
        Box::into_raw(Box::new(Board { inner: rusty_renju::board::Board::from(&history) }))
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_from_string(source: *const c_char) -> *mut Board {
    if let Some(source) = unsafe { source.as_ref() }
        && let Ok(source) = unsafe { CStr::from_ptr(source) }.to_str()
        && let Ok(board) = source.parse::<rusty_renju::board::Board>()
    {
        Box::into_raw(Box::new(Board { inner: board }))
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_to_string(board: *const Board) -> *mut c_char {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(result) = CString::new(board.inner.to_string())
    {
        result.into_raw()
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_hash_key(board: *const Board) -> u64 {
    unsafe { board.as_ref() }
        .map_or(0, |board| u64::from(board.inner.hash_key))
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_player_color(board: *const Board) -> u8 {
    unsafe { board.as_ref() }
        .map_or(0, |board| board.inner.player_color as u8)
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_stones(board: *const Board) -> u8 {
    unsafe { board.as_ref() }
        .map_or(0, |board| board.inner.stones)
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_patterns(board: *const Board) -> *mut Patterns {
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
pub extern "C" fn rusty_renju_board_pattern(board: *const Board, color: u8, pos: u8) -> u32 {
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
pub extern "C" fn rusty_renju_board_is_pos_empty(board: *const Board, pos: u8) -> bool {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(pos) = rusty_renju::notation::pos::Pos::try_from(pos)
    {
        board.inner.is_pos_empty(pos)
    } else {
        false
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_is_legal_move(board: *const Board, pos: u8) -> bool {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(pos) = rusty_renju::notation::pos::Pos::try_from(pos)
    {
        board.inner.is_legal_move(pos)
    } else {
        false
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_stone_kind(board: *const Board, pos: u8) -> u8 {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(pos) = rusty_renju::notation::pos::Pos::try_from(pos)
    {
        board.inner.stone_kind(pos).map_or(COLOR_NONE, |color| color as u8)
    } else {
        COLOR_NONE
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_set(board: *const Board, pos: u8) -> *mut Board {
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
pub extern "C" fn rusty_renju_board_unset(board: *const Board, pos: u8) -> *mut Board {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(action) = rusty_renju::notation::pos::MaybePos::try_from(pos)
    {
        let board = match action {
            rusty_renju::notation::pos::MaybePos::NONE => board.inner.pass(),
            pos => board.inner.unset(pos.unwrap()),
        }.into();

        Box::into_raw(Box::new(board))
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_free(board: *mut Board) {
    if !board.is_null() {
        unsafe { drop(Box::from_raw(board)); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_describe(board: *const Board, actions: *const u8, len: usize) -> *mut BoardDescribe {
    if let Some(board) = unsafe { board.as_ref() }
        && let Some(actions) = actions_slice(actions, len)
        && let Some(history) = history_from_actions(actions)
    {
        Box::into_raw(Box::new(BoardDescribe::from(board.inner.describe(&history))))
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

#[no_mangle]
pub extern "C" fn rusty_renju_version() -> *const c_char {
    CString::new(env!("CARGO_PKG_VERSION")).unwrap().into_raw()
}
