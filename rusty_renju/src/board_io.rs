use crate::bitfield::Bitfield;
use crate::board::Board;
use crate::board_iter::{BoardExportItem, BoardIterItem};
use crate::history::History;
use crate::impl_debug_from_display;
use crate::memo::hash_key::HashKey;
use crate::notation::color::{Color, ColorContainer};
use crate::notation::pos;
use crate::notation::pos::{MaybePos, Pos};
use crate::pattern::Pattern;
use crate::slice::Slice;
use crate::utils::str_utils::join_str_horizontally;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub const SYMBOL_BLACK: char = 'X';
pub const SYMBOL_WHITE: char = 'O';
pub const SYMBOL_EMPTY: char = '.';
pub const SYMBOL_FORBID_DOUBLE_THREE: char = '3';
pub const SYMBOL_FORBID_DOUBLE_FOUR: char = '4';
pub const SYMBOL_FORBID_OVERLINE: char = '6';

pub const HISTORY_LITERAL_SEPARATOR: &str = ",";
pub const HISTORY_LITERAL_PASS: &str = "pass";

enum BoardElement {
    Stone(Color),
    Empty
}

#[typeshare::typeshare]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BoardDescribe {
    pub hash_key: HashKey,
    pub player_color: Color,
    pub field: Vec<BoardExportItem>,
}

fn match_symbol(c: char) -> Option<BoardElement> {
    match c {
        SYMBOL_BLACK => Some(BoardElement::Stone(Color::Black)),
        SYMBOL_WHITE => Some(BoardElement::Stone(Color::White)),
        SYMBOL_EMPTY | SYMBOL_FORBID_DOUBLE_THREE | SYMBOL_FORBID_DOUBLE_FOUR | SYMBOL_FORBID_OVERLINE =>
            Some(BoardElement::Empty),
        _ => None
    }
}

fn board_iter_item_to_symbol(board: &Board, pos: Pos, item: BoardIterItem) -> String {
    match item {
        BoardIterItem::Stone(color) => char::from(color),
        BoardIterItem::Pattern(_) =>
            board.patterns.forbidden_kind(pos)
                .map(char::from)
                .unwrap_or(SYMBOL_EMPTY)
    }.to_string()
}

fn parse_board_elements(source: &str) -> Result<Vec<BoardElement>, &'static str> {
    let elements: Vec<BoardElement> = source
        .lines()
        .filter_map(|line| {
            let line = line.trim();

            if !line.starts_with(|c: char| c.is_ascii_digit()) {
                return None;
            }

            Some(
                line[line.find(|c: char| !c.is_ascii_digit())?..]
                    .chars()
                    .take(pos::BOARD_WIDTH as usize * 2)
                    .filter_map(match_symbol)
            )
        })
        .rev()
        .flatten()
        .collect();

    (elements.len() == pos::BOARD_SIZE)
        .then_some(elements)
        .ok_or("Invalid elements size.")
}

fn extract_stones_by_color(color: Color, source: &[BoardElement]) -> Box<[Pos]> {
    source.iter()
        .enumerate()
        .filter_map(|(idx, symbol)|
            match symbol {
                &BoardElement::Stone(sym_color) if sym_color == color =>
                    Some(Pos::from_index(idx as u8)),
                _ => None
            }
        )
        .collect()
}

impl Board {

    pub fn to_string_with_highlighted_move(&self, pos: Pos) -> String {
        let marker = ["[".to_string(), "]".to_string()];

        self.render_with_attributes(
            |pos, &item| board_iter_item_to_symbol(self, pos, item),
            |iter_pos, _| (iter_pos == pos).then(|| (false, marker.clone()))
        )
    }

    fn make_last_moves_marker(pair: [MaybePos; 2]) -> impl Fn(Pos, &BoardIterItem) -> Option<(bool, [String; 2])> {
        const POST_MARKER: [&str; 2] = ["[", "]"];
        const PRE_MARKER: [&str; 2] = ["|", "|"];

        move |iter_pos, _| match MaybePos::from(iter_pos) {
            pos if pos == pair[1] => Some((true, POST_MARKER.map(String::from))),
            pos if pos == pair[0] => Some((false, PRE_MARKER.map(String::from))),
            _ => None,
        }
    }

    pub fn to_string_with_last_moves(&self, pair: [MaybePos; 2]) -> String {
        self.render_with_attributes(
            |pos, &item| board_iter_item_to_symbol(self, pos, item),
            Self::make_last_moves_marker(pair)
        )
    }

    pub fn to_string_with_heatmap(&self, heatmap: [f32; pos::BOARD_SIZE], log_scale: bool) -> String {
        self.to_string_with_heatmap_and_last_moves(heatmap, log_scale, [MaybePos::NONE; 2])
    }

    pub fn to_string_with_heatmap_and_last_moves(&self, heatmap: [f32; pos::BOARD_SIZE], log_scale: bool, last_moves: [MaybePos; 2]) -> String {
        let min = heatmap.into_iter()
            .fold(f32::NAN, f32::min);

        let max = heatmap.into_iter()
            .fold(f32::NAN, f32::max);

        if min.is_nan() {
            return self.to_string_with_last_moves(last_moves);
        }

        let range = max - min;
        let log_range = (range + 1.0).ln();

        let board_string = self.render_with_attributes(
            |pos, &item| {
                let cell = board_iter_item_to_symbol(self, pos, item);

                if heatmap[pos.idx_usize()].is_finite() {
                    let value = heatmap[pos.idx_usize()];

                    let factor = if log_scale {
                        (value - min + 1.0).ln() / log_range
                    } else {
                        (value - min) / range
                    };

                    let normalized = (factor.clamp(0.0, 1.0) * (u8::MAX as f32)) as u8;

                    let r: u8 = normalized;
                    let b: u8 = u8::MAX - normalized;

                    format!("\x1b[48;2;{r};0;{b}m{cell}\x1b[0m")
                } else {
                    cell
                }
            },
            Self::make_last_moves_marker(last_moves)
        );

        format!("min={min} max={max}\n{board_string}")
    }

    pub fn render_with_attributes<T1, T2>(&self, cell: T1, marker: T2) -> String where
        T1: Fn(Pos, &BoardIterItem) -> String,
        T2: Fn(Pos, &BoardIterItem) -> Option<(bool, [String; 2])>
    {
        let content = self.iter_items()
            .collect::<Vec<_>>()
            .chunks(pos::U_BOARD_WIDTH)
            .enumerate()
            .map(|(row_idx, item_row)| {
                let content: String = item_row.iter()
                    .enumerate()
                    .map(|(col_idx, item)| {
                        let pos = Pos::from_cartesian(row_idx as u8, col_idx as u8);
                        (pos, item, cell(pos, item))
                    })
                    .fold(" ".to_string(), |acc, (pos, item, cell)| {
                        if let Some((overwrite, [left, right])) = marker(pos, item) {
                            if overwrite || acc.ends_with(' ') {
                                format!("{}{left}{cell}{right}", &acc[..acc.len() - 1]).to_string()
                            } else {
                                format!("{acc}{cell}{right}").to_string()
                            }
                        } else {
                            format!("{acc}{cell} ").to_string()
                        }
                    });

                format!("{:-2}{content}{}", row_idx + 1, row_idx + 1)
            })
            .rev()
            .collect::<Vec<_>>()
            .join("\n");

        let column_hint_content: String = ('A' .. (b'A' + pos::BOARD_WIDTH) as char)
            .flat_map(|x| [x, ' '])
            .take(pos::U_BOARD_WIDTH * 2 - 1)
            .collect();

        let column_hint = format!("   {column_hint_content}");

        format!("{column_hint}\n{content}\n{column_hint}")
    }

    pub fn to_string_with_pattern_analysis(&self) -> String {
        fn build_each_color_string(board: &Board, color: Color) -> String {
            fn render_pattern(board: &Board, color: Color, extract: fn(&Pattern) -> u32) -> String {
                board.render_with_attributes(|_, item| {
                    match item {
                        &BoardIterItem::Stone(color) => char::from(color).to_string(),
                        BoardIterItem::Pattern(pattern) => {
                            let count = extract(&pattern[color]);

                            if count > 0 {
                                count.to_string()
                            } else {
                                SYMBOL_EMPTY.to_string()
                            }
                        }
                    }
                }, |_, _| None)
            }

            let open_three = format!("open_three\n{}", render_pattern(board, color, Pattern::count_open_threes));
            let closed_four = format!("closed_four\n{}", render_pattern(board, color, Pattern::count_closed_fours));
            let open_four = format!("open_four\n{}", render_pattern(board, color, Pattern::count_open_fours));
            let close_three = format!("close_three\n{}", render_pattern(board, color, Pattern::count_close_threes));
            let five = format!("five\n{}", render_pattern(board, color, Pattern::count_fives));
            let potential = format!("potential\n{}", render_pattern(board, color, Pattern::count_potentials));

            join_str_horizontally(&[&open_three, &closed_four, &open_four, &close_three, &five, &potential])
        }

        format!(
            "player={}\n{self}\nblack\n{}\nwhite\n{}",
            self.player_color,
            build_each_color_string(self, Color::Black),
            build_each_color_string(self, Color::White)
        )
    }

    pub fn describe(&self, history: &History) -> BoardDescribe {
        BoardDescribe {
            hash_key: self.hash_key,
            player_color: self.player_color,
            field: self.export_items(history).into()
        }
    }

}

impl From<&History> for Board {
    fn from(value: &History) -> Self {
        let mut board = Board::default();

        board.batch_set_mut(value.actions());

        board
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render_with_attributes(
            |pos, &item| board_iter_item_to_symbol(self, pos, item),
            |_, _| None
        ))
    }
}

impl_debug_from_display!(Board);

impl FromStr for Board {
    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let elements = parse_board_elements(source)?;

        let blacks = extract_stones_by_color(Color::Black, &elements);
        let whites = extract_stones_by_color(Color::White, &elements);

        let mut board = Board::default();
        let player_color = Color::player_color_from_each_moves(blacks.len(), whites.len());

        board.batch_set_each_color_mut(blacks, whites, player_color);

        Ok(board)
    }
}

impl FromStr for Slice {
    type Err = &'static str;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let fields: Box<[BoardElement]> = source.chars()
            .filter_map(match_symbol)
            .collect();

        let field_len = fields.len() as u8;

        if !(5 ..= pos::BOARD_WIDTH).contains(&field_len) {
            Err("Invalid size.")
        } else {
            Ok(IntoIterator::into_iter(fields)
                .enumerate()
                .fold(
                    Slice::empty(0, field_len, 0, 0),
                    |acc, (idx, field)| {
                        match field {
                            BoardElement::Stone(color) => acc.set(color, idx as u8),
                            _ => acc
                        }
                    }
                )
            )
        }
    }
}

impl Display for Slice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let content = (0..self.length)
            .map(|idx| match self.stone_kind(idx) {
                Some(color) => char::from(color),
                None => SYMBOL_EMPTY
            })
            .map(String::from)
            .collect::<Vec<_>>()
            .join(" ");

        write!(f, "{content}")
    }
}

#[macro_export] macro_rules! board {
    ($board_str:expr) => {{
        use std::str::FromStr;

        $crate::board::Board::from_str($board_str).unwrap()
    }};
    () => {
        $crate::board::Board::default()
    };
}

#[typeshare::typeshare]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct BoardData {
    hash_key: HashKey,
    player_color: Color,
    bitfield: ColorContainer<Bitfield>,
}

#[cfg(feature = "serde")]
impl Serialize for Board {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        BoardData {
            hash_key: self.hash_key,
            player_color: self.player_color,
            bitfield: self.slices.bitfield()
        }.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Board {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let data = BoardData::deserialize(deserializer)?;

        let black_moves = data.bitfield[Color::Black].iter_hot_pos().collect::<Box<_>>();
        let white_moves = data.bitfield[Color::White].iter_hot_pos().collect::<Box<_>>();

        let mut board = Board::default();

        board.batch_set_each_color_mut(black_moves, white_moves, data.player_color);

        Ok(board)
    }
}
