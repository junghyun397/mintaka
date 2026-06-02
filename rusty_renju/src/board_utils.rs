#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typeshare")]
use typeshare::typeshare;
use crate::board::Board;
use crate::notation::color::Color;
use crate::notation::direction::Direction;
use crate::notation::pos::Pos;
use crate::notation::rule::RuleKind;
use crate::slice::Slice;

#[cfg_attr(feature = "typeshare", typeshare)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BoardWinner {
    pub color: Color,
    pub moves: [Pos; 5],
}

impl<const R: RuleKind> Board<R> {
    pub fn find_winner(&self, pos: Pos) -> Option<Color> {
        [
            Some(&self.slices.horizontal_slices[pos.row_usize()]),
            Some(&self.slices.vertical_slices[pos.col_usize()]),
            self.slices.ascending_slice(pos),
            self.slices.descending_slice(pos),
        ].iter()
            .find_map(|maybe_slice| maybe_slice
                .and_then(Slice::winner)
            )
    }

    pub fn find_global_winner(&self) -> Option<Color> {
        self.slices.horizontal_slices.iter()
            .chain(self.slices.vertical_slices.iter())
            .chain(self.slices.ascending_slices.iter())
            .chain(self.slices.descending_slices.iter())
            .find_map(Slice::winner)
    }

    pub fn find_global_winning_moves(&self) -> Option<BoardWinner> {
        self.slices.horizontal_slices.iter().map(|slice| (Direction::Horizontal, slice))
            .chain(self.slices.vertical_slices.iter().map(|slice| (Direction::Vertical, slice)))
            .chain(self.slices.ascending_slices.iter().map(|slice| (Direction::Ascending, slice)))
            .chain(self.slices.descending_slices.iter().map(|slice| (Direction::Descending, slice)))
            .find_map(|(direction, slice)| {
                slice.winner_idx::<{ Color::Black }>().map(|idx| (Color::Black, idx))
                    .or(slice.winner_idx::<{ Color::White }>().map(|idx| (Color::White, idx)))
                    .map(|(color, idx)| BoardWinner {
                        color,
                        moves: std::array::from_fn(|sequence|
                            slice.start_pos.directional_offset_unchecked(direction, idx as isize)
                                .directional_offset_unchecked(direction, sequence as isize)
                        )
                    })
            })
    }
}
