use rusty_renju::board::Board;
use rusty_renju::board_iter::BoardIterItem;
use rusty_renju::history::History;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{Pos, BOARD_WIDTH};
use tiny_skia::Pixmap;

const POINT_SIZE: u32 = 60; // must have 30 as a factor
const COORDINATE_SIZE: u32 = POINT_SIZE / 2;
const COORDINATE_START_OFFSET: u32 = COORDINATE_SIZE + POINT_SIZE / 2;
const COORDINATE_PADDING: u32 = COORDINATE_SIZE / 5;

const LINE_WEIGHT: f32 = POINT_SIZE as f32 / 30.0;
const LINE_START_POS: f32 = COORDINATE_SIZE as f32 + POINT_SIZE as f32 / 2.0 - LINE_WEIGHT / 2.0;
const LINE_END_POS: f32 = BOARD_WIDTH as f32 * POINT_SIZE as f32 - POINT_SIZE as f32 + LINE_WEIGHT;

const STONE_SIZE: f32 = POINT_SIZE as f32 - POINT_SIZE as f32 / 30.0;
const STONE_OFFSET: f32 = (POINT_SIZE as f32 - STONE_SIZE) / 2.0;

const BORDER_SIZE: f32 = POINT_SIZE as f32 / 20.0;

const LATEST_MOVE_DOT_SIZE: f32 = POINT_SIZE as f32 / 5.0;
const LATEST_MOVE_DOT_OFFSET: f32 = (POINT_SIZE as f32 - LATEST_MOVE_DOT_SIZE) / 2.0;

const LATEST_MOVE_CROSS_SIZE: f32 = POINT_SIZE as f32 / 3.0;
const LATEST_MOVE_CROSS_WEIGHT: f32 = POINT_SIZE as f32 / 30.0;
const LATEST_MOVE_CROSS_HEIGHT_OFFSET: f32 = (POINT_SIZE as f32 - LATEST_MOVE_CROSS_WEIGHT) / 2.0;
const LATEST_MOVE_CROSS_WIDTH_OFFSET: f32 = (POINT_SIZE as f32 - LATEST_MOVE_CROSS_SIZE) / 2.0;

const FORBID_DOT_SIZE: f32 = POINT_SIZE as f32 / 5.0;
const FORBID_DOT_OFFSET: f32 = (POINT_SIZE as f32 - FORBID_DOT_SIZE) / 2.0;

macro_rules! font_path { () => {"../assets/BitstreamCharter.otf"}; }

const HISTORY_FONT_SIZE: f32 = POINT_SIZE as f32 / 2.5;
const COORDINATE_FONT_SIZE: f32 = POINT_SIZE as f32 / 2.5;

struct ColorPalette {
    color_wood: tiny_skia::Color,
    color_black: tiny_skia::Color,
    color_white: tiny_skia::Color,
    color_grey: tiny_skia::Color,
    color_red: tiny_skia::Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            color_wood: tiny_skia::Color::from_rgba8(242, 176, 109, 255),
            color_black: tiny_skia::Color::from_rgba8(0, 0, 0, 255),
            color_white: tiny_skia::Color::from_rgba8(255, 255, 255, 255),
            color_grey: tiny_skia::Color::from_rgba8(54, 57, 63, 255),
            color_red: tiny_skia::Color::from_rgba8(240, 0, 0, 255),
        }
    }
}

pub enum HistoryRenderType {
    None,
    Recent,
    Sequence,
}

pub struct ImageBoardRenderer {
    color_palette: ColorPalette,
    board_width: u32,
    dimension: u32,
    background_pixmap: Pixmap,
    font_bin: Vec<u8>,
}

pub struct RenderArtifact {
    actions: usize,
    pixmap: Pixmap,
    history_layer: Option<Pixmap>,
}

impl Default for ImageBoardRenderer {
    fn default() -> Self {
        let board_width = POINT_SIZE * BOARD_WIDTH as u32;
        let pixmap_width = COORDINATE_SIZE * 2 + board_width;

        let mut background_pixmap = Pixmap::new(pixmap_width, pixmap_width).unwrap();
        Self::initialize_background(ColorPalette::default(), &mut background_pixmap, pixmap_width, board_width);

        Self {
            color_palette: ColorPalette::default(),
            board_width,
            dimension: pixmap_width,
            background_pixmap,
            font_bin: include_bytes!(font_path!()).to_vec(),
        }
    }
}

impl ImageBoardRenderer {
    fn initialize_background(color_palette: ColorPalette, pixmap: &mut Pixmap, dimension: u32, board_width: u32) {
        todo!() // draw background grid
    }

    fn pos_to_board_pos(&self, pos: Pos) -> (f32, f32) {
        let x = COORDINATE_SIZE as f32 + pos.col() as f32 * POINT_SIZE as f32;
        let y = COORDINATE_SIZE as f32 + (BOARD_WIDTH as f32 - pos.row() as f32 - 1.0) * POINT_SIZE as f32;
        (x, y)
    }

    fn render_stone(
        pixmap: &mut Pixmap,
        pos: Pos,
        color: Color,
    ) {
        todo!()
    }

    fn render_history_layer(
        history: &History,
        history_cache: Option<Pixmap>
    ) -> Pixmap {
        todo!()
    }

    pub fn render(
        &self,
        board: &Board,
        maybe_render_artifact: Option<RenderArtifact>,
        maybe_history: Option<&History>,
        maybe_offers: Option<&[Pos]>,
        maybe_blinds: Option<&[Pos]>,
        render_type: HistoryRenderType,
        display_forbidden_moves: bool,
    ) -> (Pixmap, RenderArtifact) {
        let (mut pixmap, history_cache) = if let (Some(history), Some(render_artifact))
            = (maybe_history, maybe_render_artifact)
        {
            let mut pixmap = render_artifact.pixmap;

            for (pos, color) in history.0.iter()
                .enumerate()
                .skip(render_artifact.actions)
                .filter_map(|(seq, action)|
                    if let Some(pos) = action.maybe_move() {
                        Some((pos, Color::player_color_from_moves(seq)))
                    } else {
                        None
                    }
                )
            {
                Self::render_stone(&mut pixmap, pos, color);
            }

            (pixmap, render_artifact.history_layer)
        } else {
            let mut pixmap = self.background_pixmap.clone();

            for (pos, color) in board.iter_items()
                .enumerate()
                .filter_map(|(idx, item)|
                    if let BoardIterItem::Stone(color) = item {
                        Some((Pos::from_index(idx as u8), color))
                    } else {
                        None
                    }
                )
            {
                Self::render_stone(&mut pixmap, pos, color);
            }

            (pixmap, None)
        };

        let mut pixmap_artifact = pixmap.clone();

        let mut information_layer = Pixmap::new(self.dimension, self.dimension).unwrap();

        if display_forbidden_moves {
            for (idx, _) in board.iter_items()
                .enumerate()
                .filter(|(_, item)|
                    if let BoardIterItem::Pattern(container) = item {
                        container.black.is_forbidden()
                    } else {
                        false
                    }
                )
            {
                let pos: Pos = idx.into();
                todo!() // draw on information-layer
            }
        }

        if let Some(offers) = maybe_offers {
            for offer in offers {
                todo!() // draw on information-layer
            }
        }

        if let Some(blinds) = maybe_blinds {
            for blind in blinds {
                todo!() // draw on information-layer
            }
        }

        let mut maybe_history_layer = None;

        match render_type {
            HistoryRenderType::Sequence => {
                let history_pixmap = Self::render_history_layer(
                    maybe_history.unwrap(),
                    history_cache
                );

                pixmap.draw_pixmap(
                    0, 0,
                    history_pixmap.as_ref(),
                    &tiny_skia::PixmapPaint::default(),
                    tiny_skia::Transform::identity(),
                    None,
                );

                maybe_history_layer = Some(history_pixmap);
            },
            HistoryRenderType::Recent => {
                let [pos1, pos2] = maybe_history.unwrap().recent_move_pair();

                if pos1.is_some() {
                    todo!() // draw on information-layer
                }

                if pos2.is_some() {
                    todo!() // draw on information-layer
                }
            },
            _ => {},
        }

        pixmap.draw_pixmap(
            0, 0,
            information_layer.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            tiny_skia::Transform::identity(),
            None,
        );

        if let Some(history_layer) = &maybe_history_layer {
            pixmap.draw_pixmap(
                0, 0,
                history_layer.as_ref(),
                &tiny_skia::PixmapPaint::default(),
                tiny_skia::Transform::identity(),
                None,
            );
        }

        (pixmap, RenderArtifact {
            actions: maybe_history.unwrap().len(),
            pixmap: pixmap_artifact,
            history_layer: maybe_history_layer,
        })
    }

}
