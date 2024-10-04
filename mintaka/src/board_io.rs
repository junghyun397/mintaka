use crate::board::Board;
use crate::board_iter::BoardIterItem;
use crate::game::Game;
use crate::impl_debug_by_display;
use crate::notation::color::Color;
use crate::notation::history::History;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::notation::pos::U_BOARD_WIDTH;
use crate::notation::rule::ForbiddenKind;
use crate::pattern::PatternUnit;
use crate::slice::Slice;
use crate::utils::str_utils::join_str_horizontally;
use regex_lite::Regex;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

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
    // regex: \d[\s\[](\S[\s\[\]]){N}\d
    let re: Regex = Regex::from_str(format!(r"\d[\s\[](\S[\s\[\]]){}{U_BOARD_WIDTH}{}\d", "{", "}").as_str()).unwrap();

    let elements: Box<[Option<Color>]> = re.find_iter(source)
        .map(|m| m.as_str())
        .collect::<Box<[&str]>>()
        .iter().rev()
        .flat_map(|m| m
            .chars()
            .skip(1) // 1> . . . . . 1
            .take(pos::BOARD_WIDTH as usize * 2) // 1 . . . . .< 1
        )
        .filter_map(match_symbol)
        .collect();

    if elements.len() != pos::BOARD_SIZE {
        return Err("Invalid elements size.");
    }

    Ok(elements)
}

fn extract_color_stones(source: &[Option<Color>], target_color: Color) -> Box<[Pos]> {
    source.iter()
        .enumerate()
        .filter_map(|(idx, symbol)|
            symbol.and_then(|color|
                (color == target_color)
                    .then(|| Pos::from_index(idx as u8))
            )
        )
        .collect()
}

impl Board {

    pub fn build_attribute_string<F>(&self, transform: F) -> String
    where F: Fn(&BoardIterItem) -> String
    {
        let content = self.iter_items()
            .collect::<Box<[_]>>()
            .chunks(U_BOARD_WIDTH)
            .enumerate()
            .map(|(row_idx, item_row)| {
                let content: String = item_row.iter()
                    .map(&transform)
                    .reduce(|head, tail|
                        format!("{head} {tail}")
                    )
                    .unwrap();

                format!("{:-2} {content} {}", row_idx + 1, row_idx + 1)
            })
            .rev()
            .reduce(|head, tail|
                format!("{head}\n{tail}")
            )
            .unwrap();

        let column_hint_content: String = ('A' .. (b'A' + pos::BOARD_WIDTH) as char)
            .flat_map(|x| [x, ' '])
            .take(U_BOARD_WIDTH * 2 - 1)
            .collect();

        let column_hint = format!("   {column_hint_content}");

        format!("{column_hint}\n{content}\n{column_hint}")
    }

    pub fn build_debug_string(&self) -> String {
        fn build_single_side_string(board: &Board, color: Color) -> String {
            fn render_pattern(board: &Board, color: Color, extract: fn(&PatternUnit) -> u32) -> String {
                board.build_attribute_string(|item| {
                    match item {
                        BoardIterItem::Stone(color) => char::from(*color).to_string(),
                        BoardIterItem::Pattern(pattern) => {
                            let count = extract(pattern.access_unit(color));

                            if count > 0 {
                                count.to_string()
                            } else {
                                SYMBOL_EMPTY.to_string()
                            }
                        }
                    }
                })
            }

            let open_three = format!("open_three\n{}", render_pattern(board, color, PatternUnit::count_open_threes));
            let close_three = format!("close_three\n{}", render_pattern(board, color, PatternUnit::count_close_threes));

            let closed_four = format!("closed_four\n{}", render_pattern(board, color, PatternUnit::count_closed_fours));
            let open_four = format!("open_four\n{}", render_pattern(board, color, PatternUnit::count_open_fours));
            let five = format!("five\n{}", render_pattern(board, color, PatternUnit::count_fives));

            join_str_horizontally(&[&open_three, &close_three, &closed_four, &open_four, &five])
        }

        format!(
            "{}\nblack\n{}\nwhite\n{}", self,
            build_single_side_string(self, Color::Black),
            build_single_side_string(self, Color::White)
        )
    }

}

impl Display for Board {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.build_attribute_string(|item|
            match item {
                BoardIterItem::Stone(color) => char::from(*color),
                BoardIterItem::Pattern(pattern) =>
                    pattern.forbidden_kind()
                        .map(char::from)
                        .unwrap_or(SYMBOL_EMPTY)
            }.to_string()
        ))
    }

}

impl_debug_by_display!(Board);

impl FromStr for Board {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let elements = parse_board_elements(source)?;

        let blacks = extract_color_stones(&elements, Color::Black);
        let whites = extract_color_stones(&elements, Color::White);

        let mut board = Board::default();
        let player_color = Color::player_color_from_batch_moves(blacks.len(), whites.len());

        board.batch_set_mut(blacks, whites, player_color);

        Ok(board)
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
            return Err("Invalid size.");
        }

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

impl Display for Slice {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let content = (0 .. self.length)
            .map(|idx|
                match self.stone_kind(idx) {
                    Some(color) => char::from(color),
                    None => SYMBOL_EMPTY
                }.to_string()
            )
            .reduce(|head, tail|
                format!("{head} {tail}")
            )
            .unwrap();

        write!(f, "{}", content)
    }

}

impl Display for History {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let history = self.0.iter()
            .map(|mv|
                match mv {
                    Some(pos) => pos.to_string(),
                    None => HISTORY_LITERAL_PASS.to_string()
                }
            )
            .reduce(|head, tail|
                format!("{head}{HISTORY_LITERAL_SEPARATOR} {tail}")
            )
            .unwrap();
        write!(f, "{history}")
    }

}

impl FromStr for History {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        // regex: [a-z][0-9][0-9]?
        let re: Regex = Regex::from_str(r"[a-z][0-9][0-9]?").unwrap();

        let history: Box<[Result<Pos, &str>]> = re.find_iter(source)
            .map(|m| Pos::from_str(m.as_str()))
            .collect();

        if let Some(result) = history.iter().find(|x| x.is_err()) {
            return Err(result.unwrap_err());
        }

        Ok(History(IntoIterator::into_iter(history)
            .filter_map(|r| r.ok()
                .map(Some)
            )
            .collect()
        ))
    }

}

impl From<History> for Game {

    fn from(history: History) -> Self {
        let blacks: Box<[Pos]> = history.0.iter()
            .enumerate()
            .filter_map(|(idx, pos)| pos
                .filter(|_| idx % 2 == 0)
            )
            .collect();

        let whites: Box<[Pos]> = history.0.iter()
            .enumerate()
            .filter_map(|(idx, pos)| pos
                .filter(|_| idx % 2 == 1)
            )
            .collect();

        let mut game = Game {
            board: Board::default(),
            history,
            result: None,
        };

        game.batch_set_mut(blacks, whites);

        game
    }

}

impl FromStr for Pos {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        u8::from_str(&source[1..])
            .map_err(|_| "Invalid row charter")
            .and_then(|row| {
                let col = source.chars().next().unwrap() as u8 - b'a';
                let pos = Pos::from_cartesian(row - 1 , col);
                if pos.col() < pos::BOARD_WIDTH && pos.row() < pos::BOARD_WIDTH {
                    Ok(pos)
                } else { Err("Invalid range") }
            })
    }

}

impl Display for Pos {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (self.col() + b'a') as char, self.row() + 1)
    }

}

impl_debug_by_display!(Pos);

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

impl_debug_by_display!(ForbiddenKind);

impl Display for ForbiddenKind {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from(*self))
    }

}
