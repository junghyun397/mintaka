use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::board_iter::BoardIterItem;
use crate::history::History;
use crate::impl_debug_from_display;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::{MaybePos, Pos};
use crate::notation::rule::ForbiddenKind;
use crate::pattern::Pattern;
use crate::slice::Slice;
use crate::utils::str_utils::join_str_horizontally;
use regex_lite::Regex;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::OnceLock;

const SYMBOL_BLACK: char = 'X';
const SYMBOL_WHITE: char = 'O';
const SYMBOL_EMPTY: char = '.';
const SYMBOL_FORBID_DOUBLE_THREE: char = '3';
const SYMBOL_FORBID_DOUBLE_FOUR: char = '4';
const SYMBOL_FORBID_OVERLINE: char = '6';

const HISTORY_LITERAL_SEPARATOR: &str = ",";
const HISTORY_LITERAL_PASS: &str = "PASS";

fn match_symbol(c: char) -> Option<Option<Color>> {
    match c {
        SYMBOL_BLACK => Some(Some(Color::Black)),
        SYMBOL_WHITE => Some(Some(Color::White)),
        SYMBOL_EMPTY | SYMBOL_FORBID_DOUBLE_THREE | SYMBOL_FORBID_DOUBLE_FOUR | SYMBOL_FORBID_OVERLINE =>
            Some(None),
        _ => None
    }
}

fn parse_board_elements(source: &str) -> Result<Box<[Option<Color>]>, &'static str> {
    const BOARD_WIDTH: usize = pos::U_BOARD_WIDTH;
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(||
        // regex: \d[\s\[\(](\S[\s\[\]\)]){N}\d
        format!(r"\d[\s\[](\S[\s\[\]]){}{BOARD_WIDTH}{}\d", "{", "}")
            .as_str()
            .parse::<Regex>()
            .unwrap()
    );

    let elements: Box<[Option<Color>]> = re.find_iter(source)
        .map(|m| m.as_str())
        .collect::<Box<[&str]>>()
        .iter().rev()
        .flat_map(|m| m
            .chars()
            .skip(1) // N> . . . . . N
            .take(pos::BOARD_WIDTH as usize * 2) // N . . . . .< N
        )
        .filter_map(match_symbol)
        .collect();

    if elements.len() != pos::BOARD_SIZE {
        return Err("Invalid elements size.");
    }

    Ok(elements)
}

fn extract_stones_by_color<const C: Color>(source: &[Option<Color>]) -> Box<[Pos]> {
    source.iter()
        .enumerate()
        .filter_map(|(idx, symbol)|
            symbol.and_then(|color|
                (color == C).then(|| Pos::from_index(idx as u8))
            )
        )
        .collect()
}

impl Board {

    pub fn to_string_with_move_marker(&self, pos: Pos) -> String {
        add_move_marker(self.to_string(), !self.player_color, pos, '[', ']')
    }

    pub fn to_string_with_move_marker_pair(&self, pre: Pos, post: Pos) -> String {
        let board_string = add_move_marker(self.to_string(), self.player_color, pre, '|', '|');
        add_move_marker(board_string, !self.player_color, post, '[', ']')
    }

    pub fn build_attribute_string<F>(&self, transform: F) -> String
    where F: Fn(&BoardIterItem) -> String
    {
        let content = self.iter_items()
            .collect::<Box<[_]>>()
            .chunks(pos::U_BOARD_WIDTH)
            .enumerate()
            .map(|(row_idx, item_row)| {
                let content: String = item_row.iter()
                    .map(&transform)
                    .collect::<Vec<_>>()
                    .join(" ");

                format!("{:-2} {content} {}", row_idx + 1, row_idx + 1)
            })
            .rev()
            .collect::<Vec<_>>()
            .join("\n");

        let column_hint_content: String = ('A' .. (b'A' + pos::BOARD_WIDTH) as char)
            .flat_map(|x| [x, ' '])
            .take(pos::U_BOARD_WIDTH * 2 - 1)
            .collect();

        let column_hint = format!("   {column_hint_content}");

        format!("{column_hint}\n{content}\n{column_hint}")
    }

    pub fn build_detailed_string(&self) -> String {
        fn build_each_color_string(board: &Board, color: Color) -> String {
            fn render_pattern(board: &Board, color: Color, extract: fn(&Pattern) -> u32) -> String {
                board.build_attribute_string(|item| {
                    match item {
                        &BoardIterItem::Stone(color) => char::from(color).to_string(),
                        BoardIterItem::Pattern(pattern) => {
                            let count = extract(pattern.access(color));

                            if count > 0 {
                                count.to_string()
                            } else {
                                SYMBOL_EMPTY.to_string()
                            }
                        }
                    }
                })
            }

            let open_three = format!("open_three\n{}", render_pattern(board, color, Pattern::count_open_threes));
            let closed_four = format!("closed_four\n{}", render_pattern(board, color, Pattern::count_closed_fours));
            let open_four = format!("open_four\n{}", render_pattern(board, color, Pattern::count_open_fours));
            let close_three = format!("close_three\n{}", render_pattern(board, color, Pattern::count_close_threes));
            let five = format!("five\n{}", render_pattern(board, color, Pattern::count_fives));

            join_str_horizontally(&[&open_three, &closed_four, &open_four, &close_three, &five])
        }

        format!(
            "{}\nblack\n{}\nwhite\n{}", self,
            build_each_color_string(self, Color::Black),
            build_each_color_string(self, Color::White)
        )
    }

}

impl Display for Board {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build_attribute_string(|item|
            match item {
                &BoardIterItem::Stone(color) => char::from(color),
                BoardIterItem::Pattern(pattern) =>
                    pattern.black.forbidden_kind()
                        .map(char::from)
                        .unwrap_or(SYMBOL_EMPTY)
            }.to_string()
        ))
    }

}

impl_debug_from_display!(Board);

impl FromStr for Board {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let elements = parse_board_elements(source)?;

        let blacks = extract_stones_by_color::<{ Color::Black }>(&elements);
        let whites = extract_stones_by_color::<{ Color::White }>(&elements);

        let mut board = Board::default();
        let player_color = Color::player_color_from_each_moves(blacks.len(), whites.len());

        board.batch_set_each_color_mut(blacks, whites, player_color);

        Ok(board)
    }

}

pub const BIN_BOARD_SIZE: usize = size_of::<[Bitfield; 2]>();

impl From<&Board> for [u8; BIN_BOARD_SIZE] {

    fn from(value: &Board) -> Self {
        let mut black_field = Bitfield::default();
        let mut white_field = Bitfield::default();

        for row in 0 .. pos::U_BOARD_WIDTH {
            for col in 0 .. pos::BOARD_WIDTH {
                match value.slices.horizontal_slices[row].stone_kind(col) {
                    Some(Color::Black) => { black_field.set_mut(Pos::from_cartesian(row as u8, col)) }
                    Some(Color::White) => { white_field.set_mut(Pos::from_cartesian(row as u8, col)) }
                    None => {}
                }
            }
        }

        unsafe { std::mem::transmute::<[[u8; 32]; 2], [u8; 64]>([black_field.0, white_field.0]) }
    }

}

impl FromStr for Slice {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let fields: Box<[Option<Color>]> = source.chars()
            .filter_map(match_symbol)
            .collect();

        let field_len = fields.len() as u8;

        if !(5 ..= pos::BOARD_WIDTH).contains(&field_len) {
            Err("Invalid size.")
        } else {
            Ok(IntoIterator::into_iter(fields)
                .enumerate()
                .fold(
                    Slice::empty(field_len, 0, 0),
                    |acc, (idx, field)| {
                        match field {
                            Some(color) => acc.set(color, idx as u8),
                            _ => acc
                        }
                    }
                )
            )
        }
    }

}

impl Display for Slice {

fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let content = (0..self.length)
            .map(|idx| match self.stone_kind(idx) {
                Some(color) => char::from(color),
                None => SYMBOL_EMPTY
            })
            .map(String::from)
            .collect::<Vec<_>>()
            .join(" ");

        write!(f, "{}", content)
    }
}

impl Display for History {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let history = self.iter()
            .map(|&mv|
                match mv {
                    MaybePos::NONE => HISTORY_LITERAL_PASS.to_string(),
                    pos => pos.unwrap().to_string()
                }
            )
            .collect::<Vec<_>>()
            .join(format!("{HISTORY_LITERAL_SEPARATOR}").as_str());

        write!(f, "{history}")
    }

}

impl FromStr for History {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        // regex: [a-z][0-9][0-9]?
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::from_str(r"[a-z][0-9][0-9]?").unwrap());

        let mut history = History::default();

        for result in re.find_iter(source)
            .map(|m| m.as_str().parse::<Pos>())
        {
            history.set_mut(result?);
        }

        Ok(history)
    }

}

impl FromStr for Color {

    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "black" | "b" => Ok(Color::Black),
            "white" | "w" => Ok(Color::White),
            &_ => Err("unknown color")
        }
    }

}

impl FromStr for Pos {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        source[1..].parse::<u8>()
            .map_err(|_| "invalid row charter")
            .and_then(|row| {
                let col = source.chars().next().unwrap() as u8 - b'a';
                let pos = Pos::from_cartesian(row - 1 , col);

                (pos.col() < pos::BOARD_WIDTH && pos.row() < pos::BOARD_WIDTH)
                    .then_some(pos)
                    .ok_or("invalid range")
            })
    }

}

impl Display for Pos {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (self.col() + b'a') as char, self.row() + 1)
    }

}

impl_debug_from_display!(Pos);

impl From<Color> for char {

    fn from(value: Color) -> Self {
        match value {
            Color::Black => SYMBOL_BLACK,
            Color::White => SYMBOL_WHITE
        }
    }

}

impl From<ForbiddenKind> for char {

    fn from(value: ForbiddenKind) -> Self {
        match value {
            ForbiddenKind::DoubleThree => SYMBOL_FORBID_DOUBLE_THREE,
            ForbiddenKind::DoubleFour => SYMBOL_FORBID_DOUBLE_FOUR,
            ForbiddenKind::Overline => SYMBOL_FORBID_OVERLINE
        }
    }

}

impl_debug_from_display!(ForbiddenKind);

impl Display for ForbiddenKind {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from(*self))
    }

}

impl_debug_from_display!(Bitfield);

impl Display for Bitfield {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let content = self.iter()
            .map(|is_hot|
                if is_hot { "X" } else { "." }
            )
            .collect::<Vec<_>>()
            .chunks(pos::U_BOARD_WIDTH)
            .rev()
            .map(|row| row.join(" "))
            .collect::<Vec<_>>()
            .join("\n");

        write!(f, "{}", content)
    }

}

pub fn add_move_marker(mut board_string: String, color: Color, pos: Pos, pre_marker: char, post_marker: char) -> String {
    const COL_INDEX_OFFSET: usize = 3 + pos::U_BOARD_WIDTH * 2; // row(2) + margin(1) + col(w*2) + br(1)
    const LINE_OFFSET: usize = 3 + pos::U_BOARD_WIDTH * 2 + 3; // row(2) + margin(1) + col(w*2) + row(2) + br(1)
    const LINE_BEGIN_OFFSET: usize = 2; // row(2)

    let reversed_row = pos::U_BOARD_WIDTH - 1 - pos.row_usize();
    let offset: usize = COL_INDEX_OFFSET
        + LINE_OFFSET * reversed_row
        - reversed_row.saturating_add_signed(-(pos::I_BOARD_WIDTH - 9))
        + LINE_BEGIN_OFFSET + pos.col_usize() * 2;

    board_string.replace_range(offset .. offset + 3, &format!("{pre_marker}{}{post_marker}", char::from(color)));
    board_string
}
