use std::fmt::{Debug, Display, Formatter};
use std::io::Read;
use std::str::FromStr;
use crate::board::Board;
use crate::notation::history::History;
use crate::notation::pos::Pos;
use crate::notation::rule::RuleKind;
use crate::slice::Slice;

impl Display for Board {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", todo!())
    }

}

impl FromStr for Board {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Err("Invalid format. example:\n
     3 . . .
     2 . O X
     1 . . .
       A B C\n")
    }

}

impl FromStr for Slice {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Err("Invalid format. example: X . . O O . . O . . . .")
    }

}

impl Display for Slice {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
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
        // Err("Invalid format. example: 1.h8 i9 2.i7 g7...");
        Err("History sequence has an conflict.")
    }

}

impl Into<Board> for History {

    fn into(self) -> Board {
        let mut board = Board::empty();
        board.batch_set_mut(&self.0, RuleKind::Renju);

        board
    }

}

impl FromStr for Pos {

    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let col = source.as_bytes()[0];
        u8::from_str(&source[1..])
            .map_err(|_| "row parsing failed")
            .map(|row|
                Pos::from_cartesian(row - 1, col - 97)
            )
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
