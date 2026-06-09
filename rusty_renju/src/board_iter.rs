use crate::board::Board;
use crate::notation::color::{Color, ColorContainer};
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::notation::rule::{ForbiddenKind, RuleKind};
use crate::pattern::Pattern;
use crate::{index_to_col, index_to_row};
use std::array;

#[repr(u64)]
#[derive(Copy, Clone)]
pub enum BoardIterItem {
    Stone(Color),
    Pattern(ColorContainer<Pattern>),
}

#[cfg(feature = "serde")]
#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum BoardExportItem {
    Stone(Color),
    Empty,
    Forbidden(ForbiddenKind),
}

#[cfg(not(feature = "serde"))]
#[derive(Debug, Copy, Clone)]
pub enum BoardExportItem {
    Stone(Color),
    Empty,
    Forbidden(ForbiddenKind),
}

impl<const R: RuleKind> Board<R> {
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
