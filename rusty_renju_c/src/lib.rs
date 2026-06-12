use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use rusty_renju::dispatch_any_board;
use rusty_renju::utils::empty::Empty;

const COLOR_NONE: u8 = u8::MAX;

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_color_black() -> u8 { rusty_renju::notation::color::Color::Black as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_color_white() -> u8 { rusty_renju::notation::color::Color::White as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_color_none() -> u8 { COLOR_NONE }

const RULE_KIND_RENJU: u8 = rusty_renju::notation::rule::RuleKind::Renju as u8;
const RULE_KIND_GOMOKU: u8 = rusty_renju::notation::rule::RuleKind::Gomoku as u8;
const RULE_KIND_FREESTYLE: u8 = rusty_renju::notation::rule::RuleKind::Freestyle as u8;

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_rule_renju() -> u8 { RULE_KIND_RENJU }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_rule_gomoku() -> u8 { RULE_KIND_GOMOKU }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_rule_freestyle() -> u8 { RULE_KIND_FREESTYLE }

const FORBIDDEN_KIND_NONE: u8 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_forbidden_kind_none() -> u8 { FORBIDDEN_KIND_NONE }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_forbidden_kind_double_three() -> u8 { rusty_renju::notation::rule::ForbiddenKind::DoubleThree as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_forbidden_kind_double_four() -> u8 { rusty_renju::notation::rule::ForbiddenKind::DoubleFour as u8 }
#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_forbidden_kind_overline() -> u8 { rusty_renju::notation::rule::ForbiddenKind::Overline as u8 }

const MAYBE_POS_NONE: u32 = rusty_renju::notation::pos::MaybePos::INVALID_POS.idx() as u32;

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_pos_none() -> u32 { MAYBE_POS_NONE }

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
    pub content: u8,
}

impl From<rusty_renju::board_iter::BoardExportItem> for BoardExportItem {
    fn from(value: rusty_renju::board_iter::BoardExportItem) -> Self {
        match value {
            rusty_renju::board_iter::BoardExportItem::Empty => BoardExportItem {
                kind: BOARD_EXPORT_ITEM_EMPTY,
                content: COLOR_NONE,
            },
            rusty_renju::board_iter::BoardExportItem::Stone(color) => BoardExportItem {
                kind: BOARD_EXPORT_ITEM_STONE,
                content: color as u8,
            },
            rusty_renju::board_iter::BoardExportItem::Forbidden(kind) => BoardExportItem {
                kind: BOARD_EXPORT_ITEM_FORBIDDEN,
                content: kind as u8,
            },
        }
    }
}

#[repr(C)]
pub struct BoardWinner {
    pub is_some: u8,
    pub color: u8,
    pub sequence: [u32; 5],
}

impl From<Option<rusty_renju::board_utils::BoardWinner>> for BoardWinner {
    fn from(value: Option<rusty_renju::board_utils::BoardWinner>) -> Self {
        BoardWinner {
            is_some: value.is_some() as u8,
            color: value.as_ref().map_or(COLOR_NONE, |winner|
                winner.color as u8
            ),
            sequence: value.as_ref().map_or([MAYBE_POS_NONE; 5], |winner|
                winner.moves.map(|pos| pos.idx() as u32)
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
    pub black_pattens: [u32; rusty_renju::pattern::PATTERN_SIZE],
    pub white_pattens: [u32; rusty_renju::pattern::PATTERN_SIZE],
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_empty_hash() -> u64 {
    rusty_renju::hash_key::HashKey::empty().into()
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_version() -> *const c_char {
    CString::new(env!("CARGO_PKG_VERSION")).unwrap().into_raw()
}

fn into_raw_board(
    board: rusty_renju::board_io::AnyBoard,
) -> *mut rusty_renju::board_io::AnyBoard {
    Box::into_raw(Box::new(board))
}

fn rule_kind_from_u8(rule_kind: u8) -> Option<rusty_renju::notation::rule::RuleKind> {
    match rule_kind {
        RULE_KIND_RENJU => Some(rusty_renju::notation::rule::RuleKind::Renju),
        RULE_KIND_GOMOKU => Some(rusty_renju::notation::rule::RuleKind::Gomoku),
        RULE_KIND_FREESTYLE => Some(rusty_renju::notation::rule::RuleKind::Freestyle),
        _ => None,
    }
}

fn any_board_from_string(
    rule_kind: rusty_renju::notation::rule::RuleKind,
    source: &str,
) -> Option<rusty_renju::board_io::AnyBoard> {
    Some(dispatch_any_board!(wrap rule_kind, 
        source.parse().ok()?
    ))
}

fn any_board_from_history(
    rule_kind: rusty_renju::notation::rule::RuleKind,
    actions: Vec<rusty_renju::notation::pos::MaybePos>,
) -> rusty_renju::board_io::AnyBoard {
    dispatch_any_board!(wrap rule_kind, 
        rusty_renju::board::Board::from(&actions.as_slice().into())
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_empty_board(rule_kind: u8) -> *mut rusty_renju::board_io::AnyBoard {
    if let Some(rule_kind) = rule_kind_from_u8(rule_kind) {
        into_raw_board(dispatch_any_board!(wrap rule_kind, 
            rusty_renju::board::Board::empty()
        ))
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_from_history(
    rule_kind: u8,
    actions: *const u32,
    len: usize,
) -> *mut rusty_renju::board_io::AnyBoard {
    if let Some(rule_kind) = rule_kind_from_u8(rule_kind)
        && let Some(actions)
            = rusty_renju::notation::ffi::try_from_raw_slice::<rusty_renju::notation::pos::MaybePos>(actions, len)
    {
        into_raw_board(any_board_from_history(rule_kind, actions))
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_from_string(
    rule_kind: u8,
    source: *const c_char,
) -> *mut rusty_renju::board_io::AnyBoard {
    if let Some(rule_kind) = rule_kind_from_u8(rule_kind)
        && let Some(source) = unsafe { source.as_ref() }
        && let Ok(source) = unsafe { CStr::from_ptr(source) }.to_str()
        && let Some(board) = any_board_from_string(rule_kind, source)
    {
        into_raw_board(board)
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_to_string(
    board: *const rusty_renju::board_io::AnyBoard,
) -> *mut c_char {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(result) = dispatch_any_board!(board, board => CString::new(board.to_string()))
    {
        result.into_raw()
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_set(
    board: *const rusty_renju::board_io::AnyBoard,
    pos: u32,
) -> *mut rusty_renju::board_io::AnyBoard {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(action) = rusty_renju::notation::pos::MaybePos::try_from(pos as u8)
    {
        let board = dispatch_any_board!(wrap board, board => {
            if let Some(pos) = action.ok() {
                board.set(pos)
            } else {
                board.pass()
            }
        });

        into_raw_board(board)
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_unset(
    board: *const rusty_renju::board_io::AnyBoard,
    pos: u32,
) -> *mut rusty_renju::board_io::AnyBoard {
    if let Some(board) = unsafe { board.as_ref() }
        && let Ok(maybe_pos) = rusty_renju::notation::pos::MaybePos::try_from(pos as u8)
    {
        let board = dispatch_any_board!(wrap board, board => {
            if let Some(pos) = maybe_pos.ok() {
                board.unset(pos)
            } else {
                board.pass()
            }
        });

        into_raw_board(board)
    } else {
        ptr::null_mut()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_free(
    board: *mut rusty_renju::board_io::AnyBoard,
) {
    if !board.is_null() {
        unsafe { drop(Box::from_raw(board)); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_rule_kind(
    board: *const rusty_renju::board_io::AnyBoard,
) -> u8 {
    if let Some(board) = unsafe { board.as_ref() } {
        board.rule_kind() as u8
    } else {
        u8::MAX
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_describe(
    board: *const rusty_renju::board_io::AnyBoard,
    out: *mut BoardDescribe,
) -> bool {
    if let Some(board) = unsafe { board.as_ref() }
        && !out.is_null()
    {
        unsafe { out.write(dispatch_any_board!(board, board => board.describe().into())) }

        true
    } else {
        false
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rusty_renju_board_pattens(
    board: *const rusty_renju::board_io::AnyBoard,
    out: *mut BoardPattens,
) -> bool {
    if let Some(board) = unsafe { board.as_ref() }
        && !out.is_null()
    {
        unsafe { out.write(dispatch_any_board!(board, board => BoardPattens {
            black_pattens: board.patterns.field[rusty_renju::notation::color::Color::Black]
                .map(u32::from),
            white_pattens: board.patterns.field[rusty_renju::notation::color::Color::White]
                .map(u32::from),
        })) }
        true
    } else {
        false
    }
}
