use rusty_renju::notation::pos;
use rusty_renju::notation::value::Eval;
use std::simd::i16x64;
use std::simd::num::SimdInt;

struct HeuristicCache {
    threat_eval: [Eval; pos::BOARD_SIZE],
    neighbor_eval: [Eval; pos::BOARD_SIZE],
}

impl HeuristicCache {

    fn new() -> Self {
        Self {
            threat_eval: [0; pos::BOARD_SIZE],
            neighbor_eval: [0; pos::BOARD_SIZE],
        }
    }

    fn set_mut(&mut self, pos: usize) {
    }

    fn unset_mut(&mut self, pos: usize) {
    }

    fn sum(&self) -> usize {
        let mut acc = i16x64::from_slice(&self.threat_eval[0 .. 64]);

        for i in (64 .. pos::BOARD_SIZE).step_by(64) {
            acc += i16x64::from_slice(&self.threat_eval[i .. i + 64]);
        }

        acc.reduce_sum() as usize
    }

}
