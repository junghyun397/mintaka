use crate::board::Board;
use crate::notation::color::{Color, ColorContainer};
use crate::pattern::Pattern;
use crate::{index_to_col, index_to_row};

#[repr(u64)]
#[derive(Copy, Clone)]
pub enum BoardIterItem {
    Stone(Color),
    Pattern(ColorContainer<Pattern>),
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
                        self.patterns.field.black[idx],
                        self.patterns.field.white[idx]
                    ))
                }
            )
    }

}
