use std::time::Duration;
use rusty_renju::utils::empty::Empty;
use crate::protocol::timer::Timer;
use crate::utils::monotonic_clock::MonotonicClock;

#[derive(Copy, Clone, Debug, PartialEq)]
struct TimeFactors {
    forced_move: bool,
    fail_lows: u32,
    best_move_changes: u32,
    best_move_search_share: f64,
}

impl Empty for TimeFactors {
    fn empty() -> Self {
        Self {
            forced_move: false,
            fail_lows: 0,
            best_move_changes: 0,
            best_move_search_share: 0.0,
        }
    }
}

impl TimeFactors {
    fn multiplier(&self) -> f64 {
        1.0
            + (self.forced_move as u64 as f64) * -0.8
            + self.fail_lows.min(3) as f64 * 0.15
            + self.best_move_changes as f64 * 0.2
            + (self.best_move_search_share.clamp(0.8, 1.0) - 0.8) * -0.5
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum LimitKind {
    Dynamic {
        factors: TimeFactors,
        base_soft_limit: Duration,
        soft_limit: Duration,
        turn: Duration,
    },
    Static,
    Infinite,
}

#[derive(Copy, Clone, Debug)]
pub struct TimeManager<CLK: MonotonicClock> {
    started_time: CLK,
    base_hard_limit: Duration,
    hard_limit: Duration,
    limit_kind: LimitKind,
}

impl<CLK: MonotonicClock> TimeManager<CLK> {
    pub fn init(timer: Timer, started_time: CLK) -> Self {
        match (timer.total_remaining, timer.turn, timer.increment) {
            (None, None, _) => {
                Self {
                    started_time,
                    base_hard_limit: Duration::MAX,
                    hard_limit: Duration::MAX,
                    limit_kind: LimitKind::Infinite,
                }
            },
            (None, Some(turn), _) => {
                Self {
                    started_time,
                    base_hard_limit: turn,
                    hard_limit: turn,
                    limit_kind: LimitKind::Static,
                }
            },
            (Some(total_remaining), turn, increment) => {
                let turn = turn.unwrap_or(Duration::MAX);

                let allocation = (total_remaining / 20 + increment / 2).min(turn);
                let soft_limit = multiply_clamp_duration(&allocation, 0.8, turn);

                Self {
                    started_time,
                    base_hard_limit: allocation,
                    hard_limit: allocation,
                    limit_kind: LimitKind::Dynamic {
                        factors: TimeFactors::empty(),
                        base_soft_limit: soft_limit,
                        soft_limit,
                        turn,
                    },
                }
            }
        }
    }

    pub fn update_fail_low(&mut self) {
        if let LimitKind::Dynamic { factors, base_soft_limit, soft_limit, turn } = &mut self.limit_kind {
            factors.fail_lows = (factors.fail_lows + 1).min(3);

            let multiplier = factors.multiplier();

            *soft_limit = multiply_clamp_duration(base_soft_limit, multiplier, *turn);
            self.hard_limit = multiply_clamp_duration(&self.base_hard_limit, multiplier, *turn);
        }
    }

    pub fn update_each_depth(
        &mut self,
        forced_move: bool,
        best_move_changes: u32,
        best_move_search_share: f64,
    ) {
        if let LimitKind::Dynamic { factors, base_soft_limit, soft_limit, turn } = &mut self.limit_kind {
            factors.forced_move = forced_move;
            factors.best_move_changes = best_move_changes;
            factors.best_move_search_share = best_move_search_share;

            let multiplier = factors.multiplier();

            *soft_limit = multiply_clamp_duration(base_soft_limit, multiplier, *turn);
            self.hard_limit = multiply_clamp_duration(&self.base_hard_limit, multiplier, *turn);
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.started_time.elapsed()
    }

    pub fn is_soft_limit_reached(&self) -> bool {
        if let LimitKind::Dynamic { soft_limit, .. } = self.limit_kind {
            self.started_time.elapsed() >= soft_limit
        } else {
            false
        }
    }

    pub fn is_hard_limit_reached(&self) -> bool {
        self.limit_kind != LimitKind::Infinite
            && self.started_time.elapsed() >= self.hard_limit
    }

    pub fn hard_limit(&self) -> Option<Duration> {
        (self.limit_kind != LimitKind::Infinite).then(|| self.hard_limit)
    }
}

fn multiply_clamp_duration(duration: &Duration, factor: f64, max: Duration) -> Duration {
    Duration::from_millis((duration.as_millis() as f64 * factor) as u64).min(max)
}
