use crate::board::Board;
use crate::notation::color::Color;
use crate::notation::rule::ForbiddenKind;
use crate::pattern::Pattern;
use crate::{index_to_col, index_to_row};

#[repr(u64)]
pub enum BoardIterItem {
    Stone(Color),
    Pattern(Pattern)
}

#[repr(u64)]
pub enum BoardIterVerboseItem {
    Stone(Color),
    Pattern(Pattern),
    Forbidden(ForbiddenKind),
    Empty
}

impl Board {

    pub fn iter_items(&self) -> impl Iterator<Item=BoardIterItem> + '_ {
        self.hot_field.iter()
            .enumerate()
            .map(|(idx, is_hot)|
                if is_hot {
                    BoardIterItem::Stone(
                        self.slices.horizontal_slices[index_to_row!(idx)]
                            .stone_kind(index_to_col!(idx) as u8)
                            .unwrap()
                    )
                } else {
                    BoardIterItem::Pattern(self.patterns.field[idx])
                }
            )
    }

    pub fn iter_verbose_items(&self) -> impl Iterator<Item=BoardIterVerboseItem> + '_ {
        self.hot_field.iter()
            .enumerate()
            .map(|(idx, is_hot)|
                if is_hot {
                    BoardIterVerboseItem::Stone(
                        self.slices.horizontal_slices[index_to_row!(idx)]
                            .stone_kind(index_to_col!(idx) as u8)
                            .unwrap()
                    )
                } else {
                    let pattern = self.patterns.field[idx];

                    if pattern.is_empty() {
                        BoardIterVerboseItem::Empty
                    } else {
                        pattern.forbidden_kind()
                            .map(BoardIterVerboseItem::Forbidden)
                            .unwrap_or_else(||
                                BoardIterVerboseItem::Pattern(pattern)
                            )
                    }
                }
            )
    }

}
