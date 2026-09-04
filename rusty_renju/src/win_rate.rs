use crate::notation::score::Score;

pub fn calculate_win_rate(score: Score) -> f32 {
    if score.is_win() {
        return 1.0;
    } else if score.is_lose() {
        return -1.0;
    }

    let score = score.value();

    score.signum() as f32 * ((score.abs() as f32) .ln_1p() / 10000.0_f32.ln_1p())
}
