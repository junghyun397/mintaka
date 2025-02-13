use rusty_renju::notation::value::{Depth, Score};

pub struct AspirationWindow {
    pub mid: Score,
    pub alpha: Score,
    pub beta: Score,
    pub alpha_fails: Score,
    pub beta_fails: Score
}

impl AspirationWindow {

    pub const INFINITE: Self = Self {
        mid: 0,
        alpha: Score::MIN,
        beta: Score::MAX,
        alpha_fails: 0,
        beta_fails: 0
    };

    pub fn extend_alpha(&mut self, score: Score, depth: Depth) {
        self.alpha_fails += 1;
    }

    pub fn extend_beta(&mut self, score: Score, depth: Depth) {
        self.beta_fails += 1;
    }

}
