use crate::board::Board;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::rule::ForbiddenKind;
use crate::pattern::Pattern;
use crate::slice::Slices;

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
        self.slices.iter()
            .enumerate()
            .map(|(idx, maybe_color)|
                if let Some(color) = maybe_color {
                    BoardIterItem::Stone(color)
                } else {
                    BoardIterItem::Pattern(
                        self.patterns.field[idx]
                    )
                }
            )
    }

    pub fn iter_verbose_items(&self) -> impl Iterator<Item=BoardIterVerboseItem> + '_ {
        self.slices.iter()
            .enumerate()
            .map(|(idx, maybe_color)|
                if let Some(color) = maybe_color {
                    BoardIterVerboseItem::Stone(color)
                } else {
                    let pattern = self.patterns.field[idx];

                    if pattern.is_empty() {
                        BoardIterVerboseItem::Empty
                    } else {
                        pattern.forbidden_kind()
                            .map(|kind| BoardIterVerboseItem::Forbidden(kind))
                            .unwrap_or_else(||
                                BoardIterVerboseItem::Pattern(pattern)
                            )
                    }
                }
            )
    }

}

impl Slices {

    pub fn iter(&self) -> impl Iterator<Item=Option<Color>> + '_ {
        self.horizontal_slices.iter()
            .flat_map(|slice|
                (0 .. pos::BOARD_WIDTH).into_iter()
                    .map(|col_idx|
                        slice.stone_kind(col_idx)
                    )
            )
    }

}
