use rusty_renju::board::Board;
use rusty_renju::board_iter::BoardIterItem;
use rusty_renju::history::{Action, History};
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
    prototype_pixmap: Pixmap,
    font_bin: Vec<u8>,
}

pub struct RenderArtifact {
    pixmap: Pixmap,
    history_layer: Option<Pixmap>,
}

impl Default for ImageBoardRenderer {
    fn default() -> Self {
        let board_width = POINT_SIZE * BOARD_WIDTH as u32;
        let dimension = COORDINATE_SIZE * 2 + board_width;

        let mut prototype_pixmap = Pixmap::new(dimension, dimension).unwrap();
        Self::initialize_prototype(ColorPalette::default(), &mut prototype_pixmap, dimension, board_width);

        Self {
            color_palette: ColorPalette::default(),
            board_width,
            dimension,
            prototype_pixmap,
            font_bin: include_bytes!(font_path!()).to_vec(),
        }
    }
}

impl ImageBoardRenderer {
    fn initialize_prototype(color_palette: ColorPalette, pixmap: &mut Pixmap, dimension: u32, board_width: u32) {
        todo!()
    }

    fn pos_to_board_pos(&self, pos: Pos) -> (f32, f32) {
        let x = COORDINATE_SIZE as f32 + pos.col() as f32 * POINT_SIZE as f32;
        let y = COORDINATE_SIZE as f32 + (BOARD_WIDTH as f32 - pos.row() as f32 - 1.0) * POINT_SIZE as f32;
        (x, y)
    }

    pub fn render_history_layer(
        &self,
        history: &History,
        history_cache: Option<Pixmap>
    ) -> (Pixmap, Pixmap) {
        todo!()
    }

    pub fn incremental_render(
        &self,
        board: &Board,
        history: &[Action],
        render_type: HistoryRenderType,
        display_forbidden_moves: bool,
        render_artifact: RenderArtifact,
    ) -> (Pixmap, RenderArtifact) {
        todo!()
    }

    pub fn full_render(
        &self,
        board: &Board,
        history: &[Action],
        render_type: HistoryRenderType,
        offers: Option<&[Pos]>,
        blinds: Option<&[Pos]>,
        display_forbidden_moves: bool
    ) -> (Pixmap, RenderArtifact) {
        let mut pixmap = self.prototype_pixmap.clone();

        let mut pixmap_archive = pixmap.clone();

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
                todo!()
            }
        }

        if let Some(offers) = offers {
            todo!()
        }

        if let Some(blinds) = blinds {
            todo!()
        }

        let mut history_layer = None;

        match render_type {
            HistoryRenderType::Sequence => {
                let mut history_pixmap = Pixmap::new(self.dimension, self.dimension).unwrap();

                history_layer = Some(history_layer);
            },
            HistoryRenderType::Recent => {
                todo!()
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

        if let Some(hist_layer) = history_layer {
            pixmap.draw_pixmap(
                0, 0,
                hist_layer.as_ref(),
                &tiny_skia::PixmapPaint::default(),
                tiny_skia::Transform::identity(),
                None,
            );
        }

        (pixmap, RenderArtifact {
            pixmap: pixmap_archive,
            history_layer,
        })
    }

}
