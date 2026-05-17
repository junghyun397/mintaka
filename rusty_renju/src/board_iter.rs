use crate::board::Board;
use crate::notation::color::{Color, ColorContainer};
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::notation::rule::ForbiddenKind;
use crate::pattern::Pattern;
use crate::{index_to_col, index_to_row};
use std::array;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typeshare")]
use typeshare::typeshare;

#[repr(u64)]
#[derive(Copy, Clone)]
pub enum BoardIterItem {
    Stone(Color),
    Pattern(ColorContainer<Pattern>),
}

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "BoardExportItemSchema"))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "content"))]
#[derive(Debug, Copy, Clone)]
pub enum BoardExportItem {
    Stone(Color),
    Empty,
    Forbidden(ForbiddenKind)
}

#[cfg(any())]
mod typeshare_workaround {
    use super::*;
    #[typeshare]
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type", content = "content")]
    pub enum BoardExportItemSchema {
        Stone(Color),
        Empty,
        Forbidden(ForbiddenKind)
    }
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
                    BoardIterItem::Pattern(ColorContainer::new(
                        self.patterns.field[Color::Black][idx],
                        self.patterns.field[Color::White][idx]
                    ))
                }
            )
    }

    pub fn export_items(&self) -> [BoardExportItem; pos::BOARD_SIZE] {
        array::from_fn(|idx| {
            let pos = Pos::from_index(idx as u8);

            self.stone_kind(pos)
                .map(|color|
                    BoardExportItem::Stone(color)
                )
                .or_else(||
                    self.patterns.forbidden_kind(pos)
                        .map(BoardExportItem::Forbidden)
                )
                .unwrap_or(BoardExportItem::Empty)
        })
    }

}
