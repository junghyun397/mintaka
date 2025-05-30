use rusty_renju::board::Board;
use rusty_renju::board_iter::BoardIterItem;
use rusty_renju::history::History;
use rusty_renju::notation::color::{Color, HeapColorContainer};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, Transform};

const POINT_SIZE: i32 = 60; // must have 30 as a factor
const COORDINATE_SIZE: i32 = POINT_SIZE / 2;
const COORDINATE_START_OFFSET: i32 = COORDINATE_SIZE + POINT_SIZE / 2;
const COORDINATE_PADDING: i32 = COORDINATE_SIZE / 5;

const BOARD_WIDTH: u32 = POINT_SIZE as u32 * pos::BOARD_WIDTH as u32;
pub const PIXMAP_WIDTH: u32 = COORDINATE_SIZE as u32 * 2 + BOARD_WIDTH;

const STONE_SIZE: f32 = POINT_SIZE as f32 - POINT_SIZE as f32 / 30.0;
const STONE_OFFSET: f32 = (POINT_SIZE as f32 - STONE_SIZE) / 2.0;

const BORDER_SIZE: f32 = POINT_SIZE as f32 / 20.0;

macro_rules! font_path { () => {"../assets/BitstreamCharter.otf"}; }

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

struct PixmapLut {
    background: Pixmap,
    stone: HeapColorContainer<Pixmap>,
    history: HeapColorContainer<[Pixmap; pos::BOARD_SIZE]>,
    forbidden_dot: Pixmap,
    recent_move_marker: HeapColorContainer<[Pixmap; 2]>,
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
    pixmap_lut: PixmapLut,
    font_bin: Vec<u8>,
}

pub struct RenderArtifact {
    actions: usize,
    pixmap: Pixmap,
    history_layer: Option<Pixmap>,
}

impl Default for ImageBoardRenderer {
    fn default() -> Self {

        let color_palette = ColorPalette::default();

        let pixmap_lut = PixmapLut {
            background: Self::initialize_background(&color_palette),
            stone: Self::initialize_stone_lut(&color_palette),
            history: Self::initialize_history_lut(&color_palette),
            forbidden_dot: Self::initialize_forbidden_dot(&color_palette),
            recent_move_marker: Self::initialize_recent_move_marker(&color_palette),
        };

        Self {
            color_palette,
            pixmap_lut,
            font_bin: include_bytes!(font_path!()).to_vec(),
        }
    }
}

impl ImageBoardRenderer {
    fn initialize_background(color_palette: &ColorPalette) -> Pixmap {
        const LINE_WEIGHT: f32 = POINT_SIZE as f32 / 30.0;
        const LINE_START_POS: f32 = COORDINATE_SIZE as f32 + POINT_SIZE as f32 / 2.0 - LINE_WEIGHT / 2.0;
        const LINE_END_POS: f32 = pos::BOARD_WIDTH as f32 * POINT_SIZE as f32 - POINT_SIZE as f32 + LINE_WEIGHT;

        const COORDINATE_FONT_SIZE: f32 = POINT_SIZE as f32 / 2.5;

        let mut pixmap = Pixmap::new(PIXMAP_WIDTH, PIXMAP_WIDTH).unwrap();
        pixmap.fill(color_palette.color_wood);

        let mut line_paint = Paint::default();
        line_paint.set_color(color_palette.color_black);
        line_paint.anti_alias = true; // Smoother lines

        let line_stroke = Stroke {
            width: LINE_WEIGHT,
            ..Stroke::default()
        };

        let mut path_builder = PathBuilder::new();
        for idx in 0 .. pos::BOARD_WIDTH {
            let offset = idx as f32 * POINT_SIZE as f32;

            // horizontal
            path_builder.move_to(LINE_START_POS, LINE_START_POS + offset);
            path_builder.line_to(LINE_START_POS + 0.0, LINE_START_POS + offset);

            // vertical
            path_builder.move_to(LINE_START_POS + offset, LINE_START_POS);
            path_builder.line_to(LINE_START_POS + offset, LINE_START_POS + 0.0);
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
        let center_xy = PIXMAP_WIDTH as f32 / 2.0;
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

    fn initialize_stone_lut(color_palette: &ColorPalette) -> HeapColorContainer<Pixmap> {
        todo!()
    }

    fn initialize_history_lut(color_palette: &ColorPalette) -> HeapColorContainer<[Pixmap; pos::BOARD_SIZE]> {
        const HISTORY_FONT_SIZE: f32 = POINT_SIZE as f32 / 2.5;

        let acc = {
            todo!()
        };

        for idx in 0 .. pos::BOARD_SIZE {
            todo!()
        }

        acc
    }

    fn initialize_forbidden_dot(color_palette: &ColorPalette) -> Pixmap {
        const FORBID_DOT_SIZE: f32 = POINT_SIZE as f32 / 5.0;
        const FORBID_DOT_OFFSET: f32 = (POINT_SIZE as f32 - FORBID_DOT_SIZE) / 2.0;

        todo!()
    }

    fn initialize_recent_move_marker(color_palette: &ColorPalette) -> HeapColorContainer<[Pixmap; 2]> {
        const LATEST_MOVE_DOT_SIZE: f32 = POINT_SIZE as f32 / 5.0;
        const LATEST_MOVE_DOT_OFFSET: f32 = (POINT_SIZE as f32 - LATEST_MOVE_DOT_SIZE) / 2.0;

        const LATEST_MOVE_CROSS_SIZE: f32 = POINT_SIZE as f32 / 3.0;
        const LATEST_MOVE_CROSS_WEIGHT: f32 = POINT_SIZE as f32 / 30.0;
        const LATEST_MOVE_CROSS_HEIGHT_OFFSET: f32 = (POINT_SIZE as f32 - LATEST_MOVE_CROSS_WEIGHT) / 2.0;
        const LATEST_MOVE_CROSS_WIDTH_OFFSET: f32 = (POINT_SIZE as f32 - LATEST_MOVE_CROSS_SIZE) / 2.0;
        todo!()
    }

    fn pos_into_board_pos(pos: Pos) -> (i32, i32) {
        let x = COORDINATE_SIZE + pos.col() as i32 * POINT_SIZE;
        let y = COORDINATE_SIZE + (pos::BOARD_WIDTH as i32 - pos.row() as i32 - 1) * POINT_SIZE;
        (x, y)
    }

    fn render_stone(
        &self,
        pixmap: &mut Pixmap,
        board_pos: (i32, i32),
        color: Color,
        translucent: bool
    ) {
        let mut paint = PixmapPaint::default();
        paint.opacity = if translucent {
            0.5
        } else {
            1.0
        };

        pixmap.draw_pixmap(
            board_pos.0, board_pos.1,
            self.pixmap_lut.stone.access(color).as_ref(),
            &paint,
            Transform::identity(),
            None
        );
    }

    fn render_history_layer(
        &self,
        history: &History,
        history_cache: Option<(usize, Pixmap)>,
    ) -> Pixmap {
        let (skip, mut pixmap) = history_cache.unwrap_or_else(||
            (0, Pixmap::new(POINT_SIZE as u32, POINT_SIZE as u32).unwrap())
        );

        for (idx, &action) in history.iter().enumerate().skip(skip) {
            if action != MaybePos::NONE {
                let color = Color::player_color_from_moves(idx);

                let (x, y) = Self::pos_into_board_pos(action.unwrap());

                pixmap.draw_pixmap(
                    x, y,
                    self.pixmap_lut.history.access(color)[idx].as_ref(),
                    &PixmapPaint::default(),
                    Transform::identity(),
                    None,
                );
            }
        }

        pixmap
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

            for (pos, color) in history.iter()
                .enumerate()
                .skip(render_artifact.actions)
                .filter_map(|(seq, &action)|
                    match action {
                        MaybePos::NONE => None,
                        _ => Some((action.unwrap(), Color::player_color_from_moves(seq))),
                    }
                )
            {
                self.render_stone(&mut pixmap, Self::pos_into_board_pos(pos), color, false);
            }

            (pixmap, render_artifact.history_layer.map(|history_layer| (history.len(), history_layer)))
        } else {
            let mut pixmap = self.pixmap_lut.background.clone();

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

                let (x, y) = Self::pos_into_board_pos(pos);

                todo!() // draw forbidden dot
            }
        }

        if let Some(offers) = maybe_offers {
            for &offer in offers {
                self.render_stone(&mut pixmap, Self::pos_into_board_pos(offer), Color::Black, true);
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
                    if action != MaybePos::NONE {
                        todo!() // draw "+" mark
                    }
                }

                if let Some(action) = opponent_action {
                    if action != MaybePos::NONE {
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
        for &action in reference_history.iter() {
            source_history.action_mut(action);
            match action {
                MaybePos::NONE => {
                    board.pass_mut();
                },
                pos => {
                    board.set_mut(pos.unwrap());

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
            }
        }

        match image_format {
            AnimationFormat::Gif => todo!(),
            AnimationFormat::Webp => todo!(),
        }
    }

}
