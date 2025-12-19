use crate::board::Board;
use crate::notation::color::{Color, ColorContainer};
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::notation::rule::ForbiddenKind;
use crate::pattern::Pattern;
use crate::{index_to_col, index_to_row};
use serde::{Deserialize, Serialize};

#[repr(u64)]
#[derive(Copy, Clone)]
pub enum BoardIterItem {
    Stone(Color),
    Pattern(ColorContainer<Pattern>),
}

#[typeshare::typeshare]
#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum BoardExportItem {
    Stone(Color),
    Empty,
    Forbidden(ForbiddenKind)
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

    pub fn iter_export_items(&self) -> impl Iterator<Item = BoardExportItem> + '_ {
        (0..pos::U8_BOARD_SIZE).map(|idx| {
            let pos = Pos::from_index(idx);

            self.stone_kind(pos)
                .map(BoardExportItem::Stone)
                .or_else(||
                    self.patterns.forbidden_kind(pos)
                        .map(BoardExportItem::Forbidden)
                )
                .unwrap_or(BoardExportItem::Empty)
        })
    }

}
