use rusty_renju::board::Board;
use rusty_renju::board_iter::BoardIterItem;
use rusty_renju::history::{Action, History};
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::{Pos, BOARD_WIDTH};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

const POINT_SIZE: u32 = 60; // must have 30 as a factor
const COORDINATE_SIZE: u32 = POINT_SIZE / 2;
const COORDINATE_START_OFFSET: u32 = COORDINATE_SIZE + POINT_SIZE / 2;
const COORDINATE_PADDING: u32 = COORDINATE_SIZE / 5;

const LINE_WEIGHT: f32 = POINT_SIZE as f32 / 30.0;
const LINE_START_POS: f32 = COORDINATE_SIZE as f32 + POINT_SIZE as f32 / 2.0 - LINE_WEIGHT / 2.0;
const LINE_END_POS: f32 = BOARD_WIDTH as f32 * POINT_SIZE as f32 - POINT_SIZE as f32 + LINE_WEIGHT;
const LINE_END_POS_DELTA: f32 = (BOARD_WIDTH as f32 - 1.0) * POINT_SIZE as f32;

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

pub enum ImageFormat {
    Png,
    Webp,
}

pub enum AnimationFormat {
    Gif,
    Webp,
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

        let background_pixmap = Self::initialize_background(&ColorPalette::default(), pixmap_width, board_width);

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
    fn initialize_background(color_palette: &ColorPalette, dimension: u32, board_width: u32) -> Pixmap {
        let mut pixmap = Pixmap::new(dimension, dimension).unwrap();
        pixmap.fill(color_palette.color_wood);

        let mut line_paint = Paint::default();
        line_paint.set_color(color_palette.color_black);
        line_paint.anti_alias = true; // Smoother lines

        let line_stroke = Stroke {
            width: LINE_WEIGHT,
            ..Stroke::default()
        };

        let mut path_builder = PathBuilder::new();
        for idx in 0 .. BOARD_WIDTH {
            let offset = idx as f32 * POINT_SIZE as f32;

            // horizontal
            path_builder.move_to(LINE_START_POS, LINE_START_POS + offset);
            path_builder.line_to(LINE_START_POS + LINE_END_POS_DELTA, LINE_START_POS + offset);

            // vertical
            path_builder.move_to(LINE_START_POS + offset, LINE_START_POS);
            path_builder.line_to(LINE_START_POS + offset, LINE_START_POS + LINE_END_POS_DELTA);
        }

        let grid_path = path_builder.finish().unwrap();
        pixmap.stroke_path(
            &grid_path,
            &line_paint,
            &line_stroke,
            Transform::identity(),
            None,
        );

        let center_dot_radius = LINE_WEIGHT * 3.0;
        let center_xy = dimension as f32 / 2.0;
        let mut center_dot_path_builder = PathBuilder::new();
        center_dot_path_builder.push_circle(center_xy, center_xy, center_dot_radius);
        if let Some(path) = center_dot_path_builder.finish() {
            pixmap.fill_path(
                &path,
                &line_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }

        pixmap
    }

    fn pos_into_board_pos(pos: Pos) -> (f32, f32) {
        let x = COORDINATE_SIZE as f32 + pos.col() as f32 * POINT_SIZE as f32;
        let y = COORDINATE_SIZE as f32 + (BOARD_WIDTH as f32 - pos.row() as f32 - 1.0) * POINT_SIZE as f32;
        (x, y)
    }

    fn render_stone(
        &self,
        pixmap: &mut Pixmap,
        board_pos: (f32, f32),
        color: Color,
        translucent: bool
    ) {
        let mut draw_color = match color {
            Color::Black => self.color_palette.color_black,
            Color::White => self.color_palette.color_white,
        };

        let mut border_color = self.color_palette.color_grey;

        if translucent {
            draw_color.set_alpha(0.5);
            border_color.set_alpha(0.5);
        }

        // todo!() // draw single stone
    }

    fn render_history_layer(
        &self,
        history: &History,
        history_cache: Option<Pixmap>
    ) -> Pixmap {
        todo!() // draw history layer
    }

    pub fn render(
        &self,
        board: &Board,
        maybe_render_artifact: Option<RenderArtifact>,
        maybe_history: Option<&History>,
        maybe_offers: Option<&[Pos]>,
        maybe_blinds: Option<&[Pos]>,
        render_type: HistoryRenderType,
        image_format: ImageFormat,
        display_forbidden_moves: bool,
    ) -> (Vec<u8>, RenderArtifact) {
        let (mut pixmap, history_cache) = if let (Some(history), Some(render_artifact))
            = (maybe_history, maybe_render_artifact)
        {
            let mut pixmap = render_artifact.pixmap;

            for (pos, color) in history.0.iter()
                .enumerate()
                .skip(render_artifact.actions)
                .filter_map(|(seq, action)|
                    if let Action::Move(pos) = action {
                        Some((*pos, Color::player_color_from_moves(seq)))
                    } else {
                        None
                    }
                )
            {
                self.render_stone(&mut pixmap, Self::pos_into_board_pos(pos), color, false);
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
                self.render_stone(&mut pixmap, Self::pos_into_board_pos(pos), color, false);
            }

            (pixmap, None)
        };

        let mut pixmap_artifact = pixmap.clone();

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

                todo!() // draw forbidden dot
            }
        }

        if let Some(offers) = maybe_offers {
            for offer in offers {
                self.render_stone(&mut pixmap, Self::pos_into_board_pos(*offer), Color::Black, true);
            }
        }

        if let Some(blinds) = maybe_blinds {
            for blind in blinds {
                todo!() // draw blind
            }
        }

        let mut maybe_history_layer = None;

        match render_type {
            HistoryRenderType::Sequence => {
                let history_pixmap = self.render_history_layer(
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
                let [player_action, opponent_action]
                    = maybe_history.unwrap().recent_move_pair();

                if let Some(action) = player_action {
                    if let Action::Move(pos) = action {
                        todo!() // draw "+" mark
                    }
                }

                if let Some(action) = opponent_action {
                    if let Action::Move(pos) = action {
                        todo!() // draw "." mark
                    }
                }
            },
            _ => {},
        }

        let image_data = Self::build_image(pixmap, image_format);

        (image_data, RenderArtifact {
            actions: maybe_history.map(|history| history.len()).unwrap_or(0),
            pixmap: pixmap_artifact,
            history_layer: maybe_history_layer,
        })
    }

    fn build_image(pixmap: Pixmap, image_format: ImageFormat) -> Vec<u8> {
        match image_format {
            ImageFormat::Png => pixmap.encode_png().expect("png encoding error"),
            ImageFormat::Webp => todo!(),
        }
    }

    fn build_animation(&self, reference_history: &History, image_format: AnimationFormat) -> Vec<u8> {
        let mut frames: Vec<Vec<u8>> = Vec::new();

        let mut board = Board::default();
        let mut source_history = History::default();

        let mut render_artifact = None;
        for action in &reference_history.0 {
            source_history.action_mut(*action);
            match *action {
                Action::Move(pos) => {
                    board.set_mut(pos);

                    let (frame, artifact) = self.render(
                        &board,
                        render_artifact,
                        Some(&source_history),
                        None, None,
                        HistoryRenderType::Sequence,
                        ImageFormat::Png,
                        true
                    );

                    frames.push(frame);
                    render_artifact = Some(artifact);
                }
                Action::Pass => {
                    board.pass_mut();
                }
            }
        }

        match image_format {
            AnimationFormat::Gif => todo!(),
            AnimationFormat::Webp => todo!(),
        }
    }

}
