use std::time::Duration;
use rusty_renju::utils::empty::Empty;
use crate::batch_counter::BatchCounter;
use crate::protocol::time::{TimeUnit, TimeValue};
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
        base_soft_limit: TimeValue,
        soft_limit: TimeValue,
        base_hard_limit: TimeValue,
        hard_limit: TimeValue,
        turn: TimeValue,
    },
    Static {
        turn: TimeValue,
    },
    Infinite,
}

#[derive(Copy, Clone, Debug)]
pub struct TimeManager<CLK: MonotonicClock> {
    started_time: CLK,
    time_unit: TimeUnit,
    limit_kind: LimitKind,
}

impl<CLK: MonotonicClock> TimeManager<CLK> {
    pub fn init(timer: Timer, started_time: CLK) -> Self {
        match (timer.total_remaining, timer.turn, timer.increment) {
            (None, None, _) => {
                Self {
                    started_time,
                    time_unit: timer.time_unit,
                    limit_kind: LimitKind::Infinite,
                }
            },
            (None, Some(turn), _) => {
                Self {
                    started_time,
                    time_unit: timer.time_unit,
                    limit_kind: LimitKind::Static {
                        turn,
                    },
                }
            },
            (Some(total_remaining), turn, increment) => {
                let turn = turn.unwrap_or(TimeValue::INFINITE);

                let allocation = (total_remaining / 20 + increment / 2).min(turn);
                let soft_limit = allocation.multiply_clamp(0.8, turn);

                Self {
                    started_time,
                    time_unit: timer.time_unit,
                    limit_kind: LimitKind::Dynamic {
                        factors: TimeFactors::empty(),
                        base_soft_limit: soft_limit,
                        base_hard_limit: allocation,
                        hard_limit: allocation,
                        soft_limit,
                        turn,
                    },
                }
            }
        }
    }

    pub fn update_fail_low(&mut self) {
        if let LimitKind::Dynamic { factors, base_soft_limit, soft_limit, turn, base_hard_limit, hard_limit } = &mut self.limit_kind {
            factors.fail_lows = (factors.fail_lows + 1).min(3);

            let multiplier = factors.multiplier();

            *soft_limit = base_soft_limit.multiply_clamp(multiplier, *turn);
            *hard_limit = base_hard_limit.multiply_clamp(multiplier, *turn);
        }
    }

    pub fn update_each_depth(
        &mut self,
        forced_move: bool,
        best_move_changes: u32,
        best_move_search_share: f64,
    ) {
        if let LimitKind::Dynamic { factors, base_soft_limit, soft_limit, turn, base_hard_limit, hard_limit } = &mut self.limit_kind {
            factors.forced_move = forced_move;
            factors.best_move_changes = best_move_changes;
            factors.best_move_search_share = best_move_search_share;

            let multiplier = factors.multiplier();

            *soft_limit = base_soft_limit.multiply_clamp(multiplier, *turn);
            *hard_limit = base_hard_limit.multiply_clamp(multiplier, *turn);
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.started_time.elapsed()
    }

    fn is_limit_reached(&self, limit: TimeValue, batch_counter: &BatchCounter) -> bool {
        if self.limit_kind == LimitKind::Infinite {
            return false;
        }

        match self.time_unit {
            TimeUnit::Clock => TimeValue::from_duration(self.started_time.elapsed()) >= limit,
            TimeUnit::Nodes => TimeValue::from_nodes(batch_counter.count_global()) >= limit,
        }
    }

    pub fn is_soft_limit_reached(&self, batch_counter: &BatchCounter) -> bool {
        let LimitKind::Dynamic { soft_limit, .. } = self.limit_kind else {
            return false
        };

        self.is_limit_reached(soft_limit, batch_counter)
    }

    pub fn is_hard_limit_reached(&self, batch_counter: &BatchCounter) -> bool {
        self.hard_limit().is_some_and(|limit|
            self.is_limit_reached(limit, batch_counter)
        )
    }

    pub fn hard_limit(&self) -> Option<TimeValue> {
        match self.limit_kind {
            LimitKind::Infinite => None,
            LimitKind::Static { turn } => Some(turn),
            LimitKind::Dynamic { hard_limit, .. } => Some(hard_limit),
        }
    }
}
