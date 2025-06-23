use rusty_renju::const_for;
use rusty_renju::notation::color::{Color, ColorContainer};
use rusty_renju::pattern::Pattern;

pub fn probe_score_table_lut<const C: Color>(pattern: Pattern) -> i8 {
    let idx = encode_pattern_to_score_key(pattern);
    PATTERN_SCORE_LUT.get_ref::<C>()[idx]
}

fn encode_pattern_to_score_key(pattern: Pattern) -> usize {
    let mut pattern_key = pattern.count_closed_fours() & 0b11;

    pattern_key |= (pattern.count_open_fours() & 0b11) << 2;
    pattern_key |= (pattern.count_open_threes() & 0b11) << 4;
    pattern_key |= (pattern.has_close_three() as u32) << 6;
    pattern_key |= (pattern.has_overline() as u32) * 127;

    pattern_key as usize
}

struct HeuristicPositionScores; impl HeuristicPositionScores {
    const OPEN_THREE: i8 = 15;
    const CLOSE_THREE: i8 = 5;
    const CLOSED_FOUR: i8 = 7;
    const OPEN_FOUR: i8 = 80;
    const DOUBLE_THREE_FORK: i8 = 30;
    const THREE_FOUR_FORK: i8 = 50;
    const DOUBLE_FOUR_FORK: i8 = 80;
    const DOUBLE_THREE_FORBID: i8 = 1;
    const DOUBLE_FOUR_FORBID: i8 = -2;
    const OVERLINE_FORBID: i8 = -2;
}

const PATTERN_SCORE_LUT: ColorContainer<[i8; 0b1 << 7]> = build_pattern_score_lut();

const fn build_pattern_score_lut() -> ColorContainer<[i8; 128]> {
    let mut acc = ColorContainer::new(
        [0; 0b1 << 7],
        [0; 0b1 << 7]
    );

    const fn flash_score_variants(
        color: Color,
        lut: &mut [i8; 0b1 << 7],
    ) {
        const_for!(pattern_key in 0, 0b1 << 7; {
            let closed_fours = pattern_key & 0b11;
            let open_fours = (pattern_key & 0b1100) >> 2;
            let open_threes = (pattern_key & 0b110000) >> 4;
            let close_threes = (pattern_key & 0b1000000) >> 6;

            lut[pattern_key] = match color {
                Color::Black => {
                    if pattern_key == 127 {
                        HeuristicPositionScores::OVERLINE_FORBID
                    } else if closed_fours + open_fours > 1 {
                        HeuristicPositionScores::DOUBLE_FOUR_FORBID
                    } else if open_threes > 1 {
                        HeuristicPositionScores::DOUBLE_THREE_FORBID
                    } else if open_fours == 1 {
                        HeuristicPositionScores::OPEN_FOUR
                    } else if closed_fours == 1 && open_threes == 1 {
                        HeuristicPositionScores::THREE_FOUR_FORK
                    } else if open_threes == 1 {
                        HeuristicPositionScores::OPEN_THREE
                    } else if close_threes > 0 {
                        HeuristicPositionScores::CLOSE_THREE
                    } else {
                        0
                    }
                },
                Color::White => {
                    if open_fours > 0 {
                        HeuristicPositionScores::OPEN_FOUR
                    } else if closed_fours > 1 {
                        HeuristicPositionScores::DOUBLE_FOUR_FORK
                    } else if closed_fours > 0 && open_threes > 0 {
                        HeuristicPositionScores::THREE_FOUR_FORK
                    } else if open_threes > 1 {
                        HeuristicPositionScores::DOUBLE_THREE_FORK
                    } else if open_threes == 1 {
                        HeuristicPositionScores::OPEN_THREE
                    } else if closed_fours == 1 {
                        HeuristicPositionScores::CLOSED_FOUR
                    } else if close_threes > 0 {
                        HeuristicPositionScores::CLOSE_THREE
                    } else {
                        0
                    }
                }
            }
        });
    }

    flash_score_variants(Color::Black, &mut acc.black);
    flash_score_variants(Color::White, &mut acc.white);

    acc
}
