use rusty_renju::notation::value::{Depth, Score};

pub struct AspirationWindow {
    pub mid: Score,
    pub alpha: Score,
    pub beta: Score,
    pub alpha_fails: usize,
    pub beta_fails: usize
}

const ASPIRATION_WINDOW: i32 = 5;

impl AspirationWindow {

    fn calculate_window(depth: i32) -> i32 {
        (ASPIRATION_WINDOW + (50 / depth - 3)).max(10)
    }

    pub const INFINITE: Self = Self {
        mid: 0,
        alpha: Score::MIN,
        beta: Score::MAX,
        alpha_fails: 0,
        beta_fails: 0
    };

    pub fn wrap(mid: Score) -> Self {
        let window = Self::calculate_window(mid as i32) as Score;
        Self {
            mid,
            alpha: mid - window,
            beta: mid + window,
            alpha_fails: 0,
            beta_fails: 0
        }
    }

    pub fn extend_alpha(&mut self, score: Score, depth: Depth) {
        self.alpha_fails += 1;
    }

    pub fn extend_beta(&mut self, score: Score, depth: Depth) {
        self.beta_fails += 1;
    }

}
