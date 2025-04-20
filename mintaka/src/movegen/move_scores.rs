use rusty_renju::bitfield::Bitfield;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::Pos;
use rusty_renju::utils::platform;
use rusty_renju::{cartesian_to_index, const_for, max, min};
use std::simd::Simd;
use std::slice;

#[derive(Debug, Copy, Clone)]
pub struct MoveScores {
    pub scores: [u8; 256],
}

impl Default for MoveScores {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl From<Bitfield> for MoveScores {
    fn from(value: Bitfield) -> Self {
        value.iter_hot_pos()
            .fold(Self::default(), |mut acc, pos| {
                acc.add_neighborhood_score(pos);
                acc
            })
    }
}

impl MoveScores {

    pub const EMPTY: MoveScores = MoveScores {
        scores: [0; 256],
    };

    pub fn add_neighborhood_score(&mut self, pos: Pos) {
        for row in
            pos.row().saturating_sub(2) .. (pos.row() + 2).min(pos::BOARD_WIDTH - 1)
        {
            for col in
                pos.col().saturating_sub(2) .. (pos.col() + 2).min(pos::BOARD_WIDTH - 1)
            {
                let idx = cartesian_to_index!(row, col) as usize;
                self.scores[idx] += 1;
            }
        }
    }

    pub fn remove_neighborhood_score(&mut self, pos: Pos) {
        for row in
            pos.row().saturating_sub(2) .. (pos.row() + 2).min(pos::BOARD_WIDTH - 1)
        {
            for col in
                pos.col().saturating_sub(2) .. (pos.col() + 2).min(pos::BOARD_WIDTH - 1)
            {
                let idx = cartesian_to_index!(row, col) as usize;
                self.scores[idx] = self.scores[idx].saturating_sub(1);
            }
        }
    }

    pub fn add_neighborhood_score_simd(&mut self, pos: Pos) {
        let mut ptr = self.scores.as_mut_ptr();
        let mask_ptr = NEIGHBORHOOD_SCORE_LUT[pos.idx_usize()].as_ptr();

        unsafe { for start_idx in (0 .. 256).step_by(platform::U8_LANE_N * 4) {
            let mut acc0 = Simd::<u8, { platform::U8_LANE_N }>::from_slice(slice::from_raw_parts(
                ptr.add(start_idx), platform::U8_LANE_N)
            );
            acc0 += Simd::from_slice(slice::from_raw_parts(
                mask_ptr.add(start_idx), platform::U8_LANE_N)
            );

            let mut acc1 = Simd::<u8, { platform::U8_LANE_N }>::from_slice(slice::from_raw_parts(
                ptr.add(start_idx), platform::U8_LANE_N)
            );
            acc1 += Simd::from_slice(slice::from_raw_parts(
                mask_ptr.add(start_idx + platform::U8_LANE_N), platform::U8_LANE_N)
            );

            let mut acc2 = Simd::<u8, { platform::U8_LANE_N }>::from_slice(slice::from_raw_parts(
                ptr.add(start_idx), platform::U8_LANE_N)
            );
            acc2 += Simd::from_slice(slice::from_raw_parts(
                mask_ptr.add(start_idx + (platform::U8_LANE_N * 2)), platform::U8_LANE_N)
            );

            let mut acc3 = Simd::<u8, { platform::U8_LANE_N }>::from_slice(slice::from_raw_parts(
                ptr.add(start_idx), platform::U8_LANE_N)
            );
            acc3 += Simd::from_slice(
                slice::from_raw_parts(mask_ptr.add(start_idx + (platform::U8_LANE_N * 3)), platform::U8_LANE_N)
            );
        } }
    }

}

const NEIGHBORHOOD_SCORE_LUT: [[u8; 256]; pos::BOARD_SIZE] = build_neighborhood_score_lut();

const fn build_neighborhood_score_lut() -> [[u8; 256]; pos::BOARD_SIZE] {
    let imprint_score_pattern: [[u8; 7]; 7] = [
        [1, 0, 0, 1, 0, 0, 1],
        [0, 2, 1, 2, 1, 2, 0],
        [0, 1, 2, 2, 2, 1, 0],
        [1, 2, 2, 0, 2, 2, 1],
        [0, 1, 2, 2, 2, 1, 0],
        [0, 2, 1, 2, 1, 2, 0],
        [1, 0, 0, 1, 0, 0, 1],
    ];

    let mut acc = [[0; 256]; pos::BOARD_SIZE];

    const_for!(row in 0, pos::I_BOARD_WIDTH; {
        const_for!(col in 0, pos::I_BOARD_WIDTH; {
            let row_begin = max!(row - 3, 0);
            let row_end = min!(row + 3, pos::I_BOARD_WIDTH - 1);
            let col_begin = max!(col - 3, 0);
            let col_end = min!(col + 3, pos::I_BOARD_WIDTH - 1);

            const_for!(target_row in row_begin, row_end + 1; {
                const_for!(target_col in col_begin, col_end + 1; {
                    let row_offset = target_row - row;
                    let col_offset = target_col - col;

                    let score = imprint_score_pattern[(row_offset + 3) as usize][(col_offset + 3) as usize];
                    if score != 0 {
                        let target_idx = cartesian_to_index!(target_row, target_col);
                        acc[cartesian_to_index!(row, col) as usize][target_idx as usize] = score;
                    }
                });
            });
        })
    });

    acc
}