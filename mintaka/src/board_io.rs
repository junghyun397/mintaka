use crate::board::Board;
use crate::game::Game;
use crate::notation::color::Color;
use crate::notation::history::History;
use crate::notation::pos::Pos;
use crate::notation::rule;
use crate::notation::rule::{RuleKind, U_BOARD_WIDTH};
use crate::slice::Slice;
use regex_lite::Regex;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::u8;

const SYMBOL_BLACK: char = 'O';
const SYMBOL_WHITE: char = 'X';
const SYMBOL_EMPTY: char = '.';
const SYMBOL_FORBID_DOUBLE_THREE: char = '3';
const SYMBOL_FORBID_DOUBLE_FOUR: char = '4';
const SYMBOL_FORBID_OVERLINE: char = '6';

enum FieldSymbol {
    Stone(Color),
    Empty
}

fn match_symbol(c: char) -> Option<FieldSymbol> {
    match c {
        SYMBOL_BLACK => Some(FieldSymbol::Stone(Color::Black)),
        SYMBOL_WHITE => Some(FieldSymbol::Stone(Color::White)),
        SYMBOL_EMPTY | SYMBOL_FORBID_DOUBLE_THREE | SYMBOL_FORBID_DOUBLE_FOUR | SYMBOL_FORBID_OVERLINE =>
            Some(FieldSymbol::Empty),
        _ => None
    }
}

const SYMBOL_SET: [char; 6] =
    [SYMBOL_BLACK, SYMBOL_WHITE, SYMBOL_EMPTY, SYMBOL_FORBID_DOUBLE_THREE, SYMBOL_FORBID_DOUBLE_FOUR, SYMBOL_FORBID_OVERLINE];

fn parse_board_elements(source: &str) -> Result<Vec<FieldSymbol>, &'static str> {
    // regex: \d[\s\[](\S[\s\[\]]){N}\d
    let re: Regex = Regex::from_str(format!(r"\d[\s\[](\S[\s\[\]]){U_BOARD_WIDTH}\d").as_str()).unwrap();

    let elements: Vec<FieldSymbol> = re.find_iter(source)
        .flat_map(|m| m
            .as_str()
            .chars()
            .skip(1)
            .take(rule::BOARD_WIDTH as usize * 2)
        )
        .filter_map(|x| match_symbol(x))
        .collect();

    if elements.len() != rule::BOARD_SIZE {
        return Err("Invalid elements size.");
    }

    Ok(elements)
}

fn extract_color_stones(source: &Vec<FieldSymbol>, target_color: Color) -> Vec<Pos> {
    source.iter()
        .enumerate()
        .filter_map(|(idx, symbol)| match symbol {
            FieldSymbol::Stone(color) =>
                if *color == target_color {
                    Some(Pos::from_index(idx as u8))
                } else {
                    None
                },
            _ => None
        })
        .collect()
}

impl Board {

    fn render_attribute_text<F>(&self, transform: F) -> String
    where
        F: Fn(&Board, usize) -> String
    {
        let content = Vec::from_iter(0 .. rule::BOARD_SIZE)
            .chunks(rule::U_BOARD_WIDTH)
            .enumerate()
            .map(|(row_idx, row)| {
                let prefix: String = "".into();
                let postfix: String = "".into();

                let content: String = row.iter()
                    .map(|col_idx|
                        transform(self, *col_idx)
                    )
                    .reduce(|head, tail| {
                        format!("{head} {tail}").to_owned()
                    })
                    .unwrap();

                format!("{prefix} {content} {postfix}").to_owned()
            })
            .reduce(|head, tail|
                format!("{head}\n{tail}")
            )
            .unwrap();

        let column_hint_content: String = (65u8 .. 65u8 + rule::BOARD_WIDTH).into_iter()
            .flat_map(|x| [x as char, ' '])
            .collect();

        let column_hint = format!("   {column_hint_content}");

        format!("{column_hint}\n{content}\n{column_hint}").into()
    }

}

impl Display for Board {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", todo!())
    }

}

impl FromStr for Board {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let elements = parse_board_elements(source)?;

        let blacks = extract_color_stones(&elements, Color::Black);
        let whites = extract_color_stones(&elements, Color::White);

        let mut board = Board::default();
        let player_color =
            if blacks.len() > whites.len() {
                Color::White
            } else {
                Color::Black
            };

        board.batch_set_mut(blacks, whites, player_color, RuleKind::Renju);

        Ok(board)
    }

}

impl FromStr for Slice {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let fields: Vec<FieldSymbol> = source.chars()
            .filter_map(|x| match_symbol(x))
            .collect();

        if 5 > fields.len() || fields.len() > rule::U_BOARD_WIDTH {
            return Err("Invalid size.");
        }

        Ok(fields.iter()
            .enumerate()
            .fold(
                Slice::empty(fields.len() as u8, Pos::from_index(0)),
                |acc, (idx, field)| {
                    match field {
                        FieldSymbol::Stone(color) => acc.set(*color, idx as u8),
                        _ => acc
                    }
                }
            )
        )
    }

}

impl Display for Slice {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", todo!())
    }

}

impl Display for History {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", todo!())
    }

}

impl FromStr for History {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        // regex: [a-z][0-9][0-9]?[0-9]?
        let re: Regex = Regex::from_str(r"[a-z][0-9][0-9]?[0-9]?").unwrap();

        let history: Vec<Result<Pos, &str>> = re.find_iter(source)
            .map(|m| Pos::from_str(m.as_str()))
            .collect();

        if let Some(result) = history.iter().find(|x| x.is_err()) {
            return Err(result.unwrap_err());
        }

        Ok(History(history.iter()
            .filter_map(|r| r.ok()
                .map(|pos| Some(pos))
            )
            .collect()
        ))
    }

}

impl Into<Game> for History {

    fn into(self) -> Game {
        let mut game = Game::default();

        game
    }

}

impl FromStr for Pos {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        u8::from_str(&source[1..])
            .map_err(|_| "Invalid row charter")
            .and_then(|row| {
                let col = source.chars().next().unwrap() as u8 - 'a' as u8;
                let pos = Pos::from_cartesian(row , col - 97);
                if pos.col() < rule::BOARD_WIDTH && pos.row() < rule::BOARD_WIDTH {
                    Ok(pos)
                } else { Err("Invalid range") }
            })
    }

}

impl Debug for Pos {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (self.col() + 97) as char, self.row() + 1)
    }

}

impl Display for Pos {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }

}

impl Debug for Color {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self == &Color::Black { "O" } else { "X" })
    }

}

impl Display for Color {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }

}
