use crate::board::Board;
use crate::game::Game;
use crate::notation::color::Color;
use crate::notation::history::History;
use crate::notation::pos::Pos;
use crate::notation::rule;
use crate::notation::rule::RuleKind;
use crate::slice::Slice;
use regex_lite::Regex;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::u8;

const BOARD_CHARACTER_SET: [char; 6] = ['O', 'X', '.', '3', '4', '6'];

// regex: \d[\s\[](\S[\s\[\]]){N}\d
fn filter_map_board_elements(source: &str) -> Vec<Option<Color>> {
    const RE: Regex = Regex::from_str(r"\d[\s\[](\S[\s\[\]]){15}\d").unwrap();

    todo!()
}

fn filter_map_stones<F>(source: &Vec<Option<Color>>, op: F) -> Vec<Pos>
where F: Fn((usize, &Option<Color>)) -> Option<Pos> {
    source.iter()
        .enumerate()
        .filter_map(op)
        .collect()
}

fn filter_map_stones_pair(source: &Vec<Option<Color>>) -> (Vec<Pos>, Vec<Pos>) {
    let blacks = filter_map_stones(
        source,
        |(idx, x)| x
            .and_then(|color| match color {
                Color::Black => Some(Pos::from_index(idx as u8)),
                Color::White => None,
            })
    );

    let whites = filter_map_stones(
        source,
        |(idx, x)| x
            .and_then(|color| match color {
                Color::Black => None,
                Color::White => Some(Pos::from_index(idx as u8)),
            })
    );

    (blacks, whites)
}

impl Display for Board {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", todo!())
    }

}

impl FromStr for Board {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let boards = source.split("\n")
            .flat_map(|row| row.chars()
                .skip_while(|char| *char != ' ')
            )
            .clone();

        let fields = filter_map_board_elements(source);

        if fields.len() != rule::BOARD_SIZE {
            return Err("Invalid format.");
        }

        let mut board = Board::default();

        let (blacks, whites) = filter_map_stones_pair(&fields);
        blacks.iter()
            .zip(&whites)
            .flat_map(|(black, white)| vec![black, white])
            .for_each(|pos| {
                board.set_mut(*pos, RuleKind::Renju)
            });

        Ok(board)
    }

}

impl FromStr for Slice {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let fields = filter_map_board_elements(source);

        if fields.len() < 5 || fields.len() > rule::U_BOARD_WIDTH {
            return Err("Invalid size.");
        }

        Ok(fields.iter()
            .enumerate()
            .fold(
                Slice::empty(fields.len() as u8, Pos::from_index(0)),
                |acc, (idx, field)| {
                    match field {
                        Some(color) => acc.set(*color, idx as u8),
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
        const RE: Regex = Regex::from_str(r"[a-z][0-9][0-9]?[0-9]?").unwrap();

        todo!()
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
