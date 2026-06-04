use crate::notation::score::{Score, Scores};

pub fn calculate_win_rate(score: Score) -> f32 {
    if Score::is_winning(score) {
        return 1.0;
    } else if Score::is_losing(score) {
        return -1.0;
    }

    score.signum() as f32 * ((score.abs() as f32) .ln_1p() / 10000.0_f32.ln_1p())
}
