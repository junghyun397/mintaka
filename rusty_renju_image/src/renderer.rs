/*
Converted from the Java AWT version with the help of a LLM coding agent.
Original source = https://github.com/junghyun397/GomokuBot/blob/fbfc977210fbe238ecdd9e1b172d88203951b00c/core/src/main/kotlin/core/interact/message/graphics/ImageBoardRenderer.kt
*/

use std::array;
use std::sync::OnceLock;
use fontdue::Font;
use tiny_skia::{FillRule, Paint, Path, PathBuilder, Pixmap, PixmapPaint, PremultipliedColorU8, Rect, Transform};
use rusty_renju::board::Board;
use rusty_renju::board_iter::BoardExportItem;
use rusty_renju::history::{History, MAX_HISTORY_SIZE};
use rusty_renju::notation::color::{Color, ColorContainer};
use rusty_renju::notation::pos::{Pos, BOARD_SIZE, U_BOARD_WIDTH};
use crate::{HistoryRender, RenderPayloads};
use crate::text::raster_text;

pub fn render_pixmap(board: &Board, opts: RenderPayloads) -> Pixmap {
    let res = resources();
    let mut canvas = res.base.clone();

    draw_board(&mut canvas, board, &opts, res);

    canvas
}

fn cell_center(pos: Pos) -> (u32, u32) {
    let cell = &resources().cells[pos.idx_usize()];
    (cell.center.0.round() as u32, cell.center.1.round() as u32)
}

const POINT_SIZE: u32 = 60;
const CELL_CENTER: f32 = POINT_SIZE as f32 / 2.0;
const BOARD_PIXELS: u32 = POINT_SIZE * U_BOARD_WIDTH as u32;
const COORDINATE_SIZE: u32 = POINT_SIZE / 2;
const COORDINATE_FONT_SIZE: f32 = POINT_SIZE as f32 / 3.0;
const COORDINATE_START_OFFSET: u32 = COORDINATE_SIZE + POINT_SIZE / 2;
const COORDINATE_PADDING: u32 = COORDINATE_SIZE / 5;
pub const DIMENSION: u32 = COORDINATE_SIZE * 2 + BOARD_PIXELS;

const LINE_WEIGHT: u32 = POINT_SIZE / 30;
const LINE_START_POS: u32 = COORDINATE_SIZE + POINT_SIZE / 2 - LINE_WEIGHT / 2;
const LINE_END_POS: u32 = BOARD_PIXELS - POINT_SIZE + LINE_WEIGHT;

const STONE_SIZE: u32 = POINT_SIZE - POINT_SIZE / 30;
const BORDER_SIZE: u32 = POINT_SIZE / 20;

const LATEST_MOVE_DOT_SIZE: u32 = POINT_SIZE / 5;
const LATEST_MOVE_CROSS_SIZE: u32 = POINT_SIZE / 3;
const LATEST_MOVE_CROSS_WEIGHT: u32 = POINT_SIZE / 30;
const LATEST_MOVE_CROSS_HEIGHT_OFFSET: u32 = (POINT_SIZE - LATEST_MOVE_CROSS_WEIGHT) / 2;
const LATEST_MOVE_CROSS_WIDTH_OFFSET: u32 = (POINT_SIZE - LATEST_MOVE_CROSS_SIZE) / 2;

const FORBID_DOT_SIZE: u32 = POINT_SIZE / 5;

const HISTORY_FONT_SIZE: f32 = POINT_SIZE as f32 / 2.0;

fn circle(cx: f32, cy: f32, r: f32) -> Path {
    PathBuilder::from_circle(cx, cy, r).unwrap()
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_xywh(x, y, w, h).unwrap()
}

struct Palette {
    wood: tiny_skia::Color,
    black: tiny_skia::Color,
    white: tiny_skia::Color,
    red: tiny_skia::Color,
    grey: tiny_skia::Color,
    grey_100: tiny_skia::Color,
    black_100: tiny_skia::Color,
    white_100: tiny_skia::Color,
    black_px: PremultipliedColorU8,
    white_px: PremultipliedColorU8,
}

struct Cell {
    origin: (f32, f32),
    center: (f32, f32),
}

pub struct TextBitmap {
    pub width: u32,
    pub height: u32,
    pub baseline: f32,
    pub data: Vec<u8>,
}

struct GlyphPixmap {
    pixmap: Option<Pixmap>,
    width: u32,
    baseline: f32,
}

struct Lut {
    stone: ColorContainer<Pixmap>,
    offer: ColorContainer<Pixmap>,
    blind: Pixmap,
    forbidden: Pixmap,
    last_dot: ColorContainer<Pixmap>,
    last_cross: ColorContainer<Pixmap>,
    number: ColorContainer<Vec<GlyphPixmap>>,
}

struct Resources {
    base: Pixmap,
    cells: [Cell; BOARD_SIZE],
    lut: Lut,
}

fn resources() -> &'static Resources {
    static RES: OnceLock<Resources> = OnceLock::new();
    RES.get_or_init(Resources::new)
}

impl Resources {
    fn new() -> Self {
        let font = Font::from_bytes(
            include_bytes!("../assets/BitstreamCharter.otf") as &[u8],
            fontdue::FontSettings::default(),
        ).unwrap();

        let palette = build_palette();
        let cells = build_cells();
        let base = build_base_layer(&font, &palette);
        let lut = build_lut(&font, &palette);

        Self { base, cells, lut }
    }
}

fn build_palette() -> Palette {
    let black = tiny_skia::Color::from_rgba8(0, 0, 0, 0xFF);
    let white = tiny_skia::Color::from_rgba8(0xFF, 0xFF, 0xFF, 0xFF);

    Palette {
        wood: tiny_skia::Color::from_rgba8(0xF2, 0xB0, 0x6D, 0xFF),
        black, white,
        red: tiny_skia::Color::from_rgba8(0xF0, 0, 0, 0xFF),
        grey: tiny_skia::Color::from_rgba8(0x36, 0x39, 0x3F, 0xFF),
        grey_100: tiny_skia::Color::from_rgba8(0x36, 0x39, 0x3F, 100),
        black_100: tiny_skia::Color::from_rgba8(0, 0, 0, 100),
        white_100: tiny_skia::Color::from_rgba8(0xFF, 0xFF, 0xFF, 180),
        black_px: black.premultiply().to_color_u8(),
        white_px: white.premultiply().to_color_u8(),
    }
}

fn build_cells() -> [Cell; BOARD_SIZE] {
    array::from_fn(|idx| {
        let pos = Pos::from_index(idx as u8);
        let x = COORDINATE_SIZE as f32 + pos.col() as f32 * POINT_SIZE as f32;
        let y = COORDINATE_SIZE as f32 + (U_BOARD_WIDTH as f32 - pos.row() as f32 - 1.0) * POINT_SIZE as f32;

        Cell {
            origin: (x, y),
            center: (x + POINT_SIZE as f32 / 2.0, y + POINT_SIZE as f32 / 2.0),
        }
    })
}

fn build_base_layer(font: &Font, palette: &Palette) -> Pixmap {
    let coord_letters: [TextBitmap; U_BOARD_WIDTH] = array::from_fn(|idx| {
        let c = (b'A' + idx as u8) as char;
        raster_text(font, &c.to_string(), COORDINATE_FONT_SIZE)
    });

    let coord_rows: [(TextBitmap, TextBitmap); U_BOARD_WIDTH] = array::from_fn(|idx| {
        let row = U_BOARD_WIDTH as u32 - idx as u32;
        (
            raster_text(font, &format!("{row:>2}"), COORDINATE_FONT_SIZE),
            raster_text(font, &row.to_string(), COORDINATE_FONT_SIZE),
        )
    });

    let row_width = coord_rows.iter().map(|(left, _)| left.width).max().unwrap_or(0);

    let mut pixmap = Pixmap::new(DIMENSION, DIMENSION).unwrap();
    pixmap.fill(palette.wood);

    let mut paint = Paint::default();
    paint.anti_alias = false;
    paint.set_color(palette.black);

    for idx in 0..U_BOARD_WIDTH as u32 {
        let pos = LINE_START_POS + idx * POINT_SIZE;
        let v = rect(pos as f32, LINE_START_POS as f32, LINE_WEIGHT as f32, LINE_END_POS as f32);
        let h = rect(LINE_START_POS as f32, pos as f32, LINE_END_POS as f32, LINE_WEIGHT as f32);
        pixmap.fill_rect(v, &paint, Transform::identity(), None);
        pixmap.fill_rect(h, &paint, Transform::identity(), None);
    }

    for idx in 0..U_BOARD_WIDTH {
        let col = &coord_letters[idx];
        let col_x = COORDINATE_START_OFFSET as f32 + idx as f32 * POINT_SIZE as f32
            - col.width as f32 / 2.0;

        for y in [COORDINATE_FONT_SIZE, DIMENSION as f32 - COORDINATE_PADDING as f32] {
            draw_text(&mut pixmap, col, col_x, y, palette.black_px);
        }

        let row_y =
            COORDINATE_START_OFFSET as f32 + idx as f32 * POINT_SIZE as f32 + COORDINATE_PADDING as f32;
        let (row_left, row_right) = &coord_rows[idx];

        let left_x =
            COORDINATE_PADDING as f32 + row_width as f32 - row_left.width as f32;
        let right_x =
            DIMENSION as f32 - COORDINATE_PADDING as f32 - row_right.width as f32;

        for (x, row) in [(left_x, row_left), (right_x, row_right)] {
            draw_text(&mut pixmap, row, x, row_y, palette.black_px);
        }
    }

    let dot_radius = (LINE_WEIGHT * 3) as f32;
    let path = circle(DIMENSION as f32 / 2.0, DIMENSION as f32 / 2.0, dot_radius);
    paint.anti_alias = true;
    pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);

    pixmap
}

fn build_lut(font: &Font, palette: &Palette) -> Lut {
    fn by_color<F: FnMut(Color) -> Pixmap>(mut f: F) -> ColorContainer<Pixmap> {
        ColorContainer::new(f(Color::Black), f(Color::White))
    }

    let cell = || Pixmap::new(POINT_SIZE, POINT_SIZE).unwrap();

    let stone_like = |outer, inner_black, inner_white| {
        by_color(|color| {
            let mut pixmap = cell();
            let mut paint = Paint::default();
            paint.anti_alias = true;

            let outer_path = circle(CELL_CENTER, CELL_CENTER, STONE_SIZE as f32 / 2.0);
            paint.set_color(outer);
            pixmap.fill_path(&outer_path, &paint, FillRule::Winding, Transform::identity(), None);

            let inner_path = circle(
                CELL_CENTER,
                CELL_CENTER,
                STONE_SIZE as f32 / 2.0 - BORDER_SIZE as f32,
            );
            paint.set_color(match color { Color::Black => inner_black, Color::White => inner_white });
            pixmap.fill_path(&inner_path, &paint, FillRule::Winding, Transform::identity(), None);
            pixmap
        })
    };

    let marker = |draw: fn(&mut Pixmap, &mut Paint)| {
        by_color(|color| {
            let mut pixmap = cell();
            let mut paint = Paint::default();
            paint.set_color(match color { Color::Black => palette.white, Color::White => palette.black });
            draw(&mut pixmap, &mut paint);
            pixmap
        })
    };

    let blind = {
        let mut pixmap = cell();
        let mut paint = Paint::default();
        paint.set_color(palette.black_100);
        pixmap.fill_rect(
            rect(0.0, 0.0, POINT_SIZE as f32, POINT_SIZE as f32),
            &paint,
            Transform::identity(),
            None,
        );
        pixmap
    };

    let forbidden = {
        let mut pixmap = cell();
        let mut paint = Paint::default();
        paint.anti_alias = true;
        paint.set_color(palette.red);
        let dot = circle(CELL_CENTER, CELL_CENTER, FORBID_DOT_SIZE as f32 / 2.0);
        pixmap.fill_path(&dot, &paint, FillRule::Winding, Transform::identity(), None);
        pixmap
    };

    let last_dot = marker(|pixmap, paint| {
        paint.anti_alias = true;
        let dot = circle(CELL_CENTER, CELL_CENTER, LATEST_MOVE_DOT_SIZE as f32 / 2.0);
        pixmap.fill_path(&dot, paint, FillRule::Winding, Transform::identity(), None);
    });

    let last_cross = marker(|pixmap, paint| {
        let horizontal = rect(
            LATEST_MOVE_CROSS_WIDTH_OFFSET as f32,
            LATEST_MOVE_CROSS_HEIGHT_OFFSET as f32,
            LATEST_MOVE_CROSS_SIZE as f32,
            LATEST_MOVE_CROSS_WEIGHT as f32,
        );

        let vertical = rect(
            LATEST_MOVE_CROSS_HEIGHT_OFFSET as f32,
            LATEST_MOVE_CROSS_WIDTH_OFFSET as f32,
            LATEST_MOVE_CROSS_WEIGHT as f32,
            LATEST_MOVE_CROSS_SIZE as f32,
        );

        pixmap.fill_rect(horizontal, paint, Transform::identity(), None);
        pixmap.fill_rect(vertical, paint, Transform::identity(), None);
    });

    Lut {
        stone: stone_like(palette.grey, palette.black, palette.white),
        offer: stone_like(palette.grey_100, palette.black_100, palette.white_100),
        blind,
        forbidden,
        last_dot,
        last_cross,
        number: build_number_lut(font, palette),
    }
}

fn build_number_lut(font: &Font, palette: &Palette) -> ColorContainer<Vec<GlyphPixmap>> {
    let numbers: Vec<_> = (0..=MAX_HISTORY_SIZE)
        .map(|idx| {
            if idx == 0 {
                TextBitmap { width: 0, height: 0, baseline: 0.0, data: Vec::new() }
            } else {
                raster_text(font, &idx.to_string(), HISTORY_FONT_SIZE)
            }
        })
        .collect();
    let build = |color| numbers.iter().map(|glyph| glyph_pixmap(glyph, color)).collect();
    ColorContainer::new(build(palette.black_px), build(palette.white_px))
}

fn glyph_pixmap(glyph: &TextBitmap, color: PremultipliedColorU8) -> GlyphPixmap {
    if glyph.width == 0 || glyph.height == 0 {
        return GlyphPixmap { pixmap: None, width: 0, baseline: glyph.baseline };
    }

    let mut pixmap = Pixmap::new(glyph.width, glyph.height).unwrap();
    draw_text(&mut pixmap, glyph, 0.0, glyph.baseline, color);
    GlyphPixmap { pixmap: Some(pixmap), width: glyph.width, baseline: glyph.baseline }
}

fn blit(canvas: &mut Pixmap, sprite: &Pixmap, x: f32, y: f32) {
    let _ = canvas.draw_pixmap(
        x.round() as i32,
        y.round() as i32,
        sprite.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
}

fn blit_cell(canvas: &mut Pixmap, sprite: &Pixmap, cell: &Cell) {
    blit(canvas, sprite, cell.origin.0, cell.origin.1);
}

fn draw_positions(canvas: &mut Pixmap, res: &Resources, sprite: &Pixmap, positions: &[Pos]) {
    for pos in positions {
        blit_cell(canvas, sprite, &res.cells[pos.idx_usize()]);
    }
}

fn draw_board(canvas: &mut Pixmap, board: &Board, opts: &RenderPayloads, res: &Resources) {
    for (cell, item) in res.cells.iter().zip(board.export_items(&opts.history)) {
        match item {
            BoardExportItem::Stone(stone) => blit_cell(canvas, &res.lut.stone[stone.color], cell),
            BoardExportItem::Forbidden(_) if opts.enable_forbidden => blit_cell(canvas, &res.lut.forbidden, cell),
            _ => {}
        }
    }

    let offer = &res.lut.offer[board.player_color];
    draw_positions(canvas, res, offer, opts.offers);
    draw_positions(canvas, res, &res.lut.blind, opts.blinds);

    match opts.history_render {
        HistoryRender::Sequence => draw_sequence(canvas, &opts.history, res),
        HistoryRender::Pair => draw_recent(canvas, &opts.history, board, res, true),
        HistoryRender::Last => draw_recent(canvas, &opts.history, board, res, false),
        HistoryRender::None => {}
    }
}

fn draw_recent(
    canvas: &mut Pixmap,
    history: &History,
    board: &Board,
    res: &Resources,
    include_prev: bool,
) {
    let [prev, last] = history.recent_action_pair();

    let mut draw = |pos: Option<Pos>, sprite: &ColorContainer<Pixmap>| {
        if let Some(pos) = pos
            && let Some(stone_color) = board.stone_kind(pos)
        {
            blit_cell(canvas, &sprite[stone_color], &res.cells[pos.idx_usize()]);
        }
    };

    draw(last.into(), &res.lut.last_dot);

    if include_prev {
        draw(prev.into(), &res.lut.last_cross);
    }
}

fn draw_sequence(canvas: &mut Pixmap, history: &History, res: &Resources) {
    for (idx, entry) in history.iter().enumerate() {
        let Some(pos) = Option::<Pos>::from(*entry) else { continue };
        let seq = idx + 1;
        let stone_color = Color::player_color_from_moves(idx);
        let lut = &res.lut.number[!stone_color];
        let text = lut.get(seq).unwrap_or_else(|| lut.last().unwrap());
        let Some(sprite) = text.pixmap.as_ref() else { continue };

        let (center_x, center_y) = res.cells[pos.idx_usize()].center;
        let baseline = center_y + HISTORY_FONT_SIZE / 2.0 - HISTORY_FONT_SIZE / 5.0;
        let text_x = center_x - text.width as f32 / 2.0;

        blit(canvas, sprite, text_x, baseline - text.baseline);
    }
}

fn draw_text(canvas: &mut Pixmap, glyph: &TextBitmap, x: f32, baseline: f32, color: PremultipliedColorU8) {
    if glyph.data.is_empty() {
        return;
    }

    let (width, height) = (canvas.width() as i32, canvas.height() as i32);
    let stride = canvas.width() as usize;
    let offset_x = x.round() as i32;
    let offset_y = (baseline - glyph.baseline).round() as i32;
    let data = canvas.data_mut();
    let [sr, sg, sb, sa] = [
        color.red() as u16,
        color.green() as u16,
        color.blue() as u16,
        color.alpha() as u16,
    ];

    for (gy, row) in glyph.data.chunks(glyph.width as usize).enumerate() {
        let py = offset_y + gy as i32;
        if !(0..height).contains(&py) {
            continue;
        }

        for (gx, &mask) in row.iter().enumerate() {
            let px = offset_x + gx as i32;
            if mask == 0 || !(0..width).contains(&px) {
                continue;
            }

            let idx = (py as usize * stride + px as usize) * 4;
            let alpha = (sa * mask as u16 + 127) / 255;
            let inv = 255 - alpha;

            data[idx] = ((data[idx] as u16 * inv + 127) / 255 + (sr * mask as u16 + 127) / 255) as u8;
            data[idx + 1] = ((data[idx + 1] as u16 * inv + 127) / 255 + (sg * mask as u16 + 127) / 255) as u8;
            data[idx + 2] = ((data[idx + 2] as u16 * inv + 127) / 255 + (sb * mask as u16 + 127) / 255) as u8;
            data[idx + 3] = ((data[idx + 3] as u16 * inv + 127) / 255 + alpha) as u8;
        }
    }
}

pub fn demultiply(pixmap: &Pixmap) -> Vec<u8> {
    pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect()
}
