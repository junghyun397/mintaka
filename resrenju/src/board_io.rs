use std::fmt::{Debug, Display, Formatter};

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

impl TryFrom<&str> for Board {

    type Error = &'static str;

    fn try_from(source: &str) -> Result<Self, Self::Error> {
        Err("Invalid format. example:\n
     3 . . .
     2 . O X
     1 . . .
       A B C\n")
    }

}

impl TryFrom<&str> for Slice {

    type Error = &'static str;

    fn try_from(source: &str) -> Result<Self, Self::Error> {
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

impl TryFrom<&str> for History {

    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Err("Invalid format. example: 1.h8 2.i9 3.i7 ...");
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

impl TryFrom<&str> for Pos {

    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        todo!()
    }

}

impl Debug for Pos {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (self.col() + 97) as char, self.row() + 1)
    }

}
