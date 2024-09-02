use crate::board::Board;
use crate::formation::Formation;
use crate::notation::color::Color;
use crate::notation::forbidden_kind::ForbiddenKind;
use crate::notation::pos::Pos;
use crate::notation::rule;

pub enum BoardIterItem {
    Stone(Color),
    Formation(Formation)
}

pub enum BoardIterTaggedItem {
    Stone(Color),
    Formation(Formation),
    Forbidden(ForbiddenKind),
    Empty
}

impl Board {

    pub fn iter_items(&self) -> impl Iterator<Item =BoardIterItem> + '_ {
        self.slices.horizontal_slices.iter()
            .enumerate()
            .flat_map(move |(row_idx, row)|
                (0 .. rule::BOARD_WIDTH).into_iter()
                    .map(move |col_idx| {
                        let pos = Pos::from_cartesian(row_idx as u8, col_idx);
                        if row.black_stone_at(col_idx) {
                            return BoardIterItem::Stone(Color::Black)
                        } else if row.white_stone_at(col_idx) {
                            return BoardIterItem::Stone(Color::White)
                        } else {
                            BoardIterItem::Formation(self.formations.0[pos.idx_usize()])
                        }
                    })
            )
    }

    pub fn iter_tagged_items(&self) -> impl Iterator<Item =BoardIterTaggedItem> + '_ {
        self.slices.horizontal_slices.iter()
            .enumerate()
            .flat_map(move |(row_idx, row)|
                (0 .. rule::BOARD_WIDTH).into_iter()
                    .map(move |col_idx| {
                        let pos = Pos::from_cartesian(row_idx as u8, col_idx);
                        if row.black_stone_at(col_idx) {
                            return BoardIterTaggedItem::Stone(Color::Black)
                        } else if row.white_stone_at(col_idx) {
                            return BoardIterTaggedItem::Stone(Color::White)
                        }

                        let formation = self.formations.0[pos.idx_usize()];

                        if !formation.is_empty() {
                            formation.forbidden_kind()
                                .map(|kind| BoardIterTaggedItem::Forbidden(kind))
                                .unwrap_or_else(||
                                    BoardIterTaggedItem::Formation(formation)
                                )
                        } else {
                            BoardIterTaggedItem::Empty
                        }
                    })
            )
    }

}
