use crate::batch_counter::BatchCounter;
use crate::config::{Config, SearchObjective};
use crate::eval::evaluator::Evaluator;
use crate::game_state::RecoveryState;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TTView;
use crate::params;
use crate::principal_variation::PrincipalVariation;
use crate::thread_type::ThreadType;
use crate::utils::depth;
use crate::utils::depth::Depth;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::{MaybeScore, Score};
use rusty_renju::notation::pos;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub const KILLER_MOVE_SLOTS: usize = 2;

#[derive(Debug, Copy, Clone)]
pub struct SearchFrame {
    pub pos: MaybePos,
    pub evaluator_eval: MaybeScore,
    pub static_eval: Score,
    pub on_pv: bool,
    pub recovery_state: RecoveryState,
    pub searching: MaybePos,
}

impl SearchFrame {
    pub const EMPTY: Self = Self {
        pos: MaybePos::NONE,
        evaluator_eval: MaybeScore::NONE,
        static_eval: Score::DRAW,
        on_pv: false,
        recovery_state: RecoveryState::EMPTY,
        searching: MaybePos::NONE,
    };
}

#[derive(Clone, Copy)]
pub struct DebugStatics {
    pub visited_nodes: u32,
    pub sum_cutoff_distance: u32,
    pub sum_tt_hit: u32,
    pub sum_tt_cutoff: u32,
}

impl DebugStatics {
    const EMPTY: Self = Self {
        visited_nodes: 0,
        sum_cutoff_distance: 0,
        sum_tt_hit: 0,
        sum_tt_cutoff: 0,
    };
}

#[derive(Clone)]
pub struct ThreadData<'a, const R: RuleKind, TH: ThreadType, E: Evaluator<R>> {
    pub thread_type: TH,
    pub search_objective: SearchObjective,
    pub tid: u32,
    pub config: Config,

    pub evaluator: E,

    pub tt: TTView<'a>,
    pub ht: Box<HistoryTable>,
    pub ss: Box<[SearchFrame; depth::MAX_PLY_SLOTS]>,
    pub pvs: Box<[PrincipalVariation; depth::MAX_PLY_SLOTS]>,
    pub killers: Box<[[MaybePos; KILLER_MOVE_SLOTS]; depth::MAX_PLY_SLOTS]>,
    pub debug_statics: Box<[DebugStatics; depth::MAX_PLY_SLOTS]>,

    pub lmr_table: Box<[[Depth; depth::MAX_PLY_SLOTS]; 64]>,

    pub root_pv: PrincipalVariation,
    pub root_moves_in_1k: [u32; pos::BOARD_SIZE],
    pub singular_root: bool,

    pub batch_counter: BatchCounter<'a>,
    aborted: &'a AtomicBool,

    pub best_move: MaybePos,
    pub selective_depth: Depth,

    pub ply: usize,
}

impl<'a, const R: RuleKind, TH: ThreadType, E: Evaluator<R>> ThreadData<'a, R, TH, E> {
    pub fn new(
        thread_type: TH, tid: u32,
        search_objective: SearchObjective,
        config: Config,
        evaluator: E,
        tt: TTView<'a>,
        ht: HistoryTable,
        aborted: &'a AtomicBool,
        global_counter_in_1k: &'a AtomicU32
    ) -> Self {
        Self {
            thread_type,
            search_objective,
            tid,
            config,
            tt,
            evaluator,
            ht: Box::new(ht),
            ss: Box::new([SearchFrame::EMPTY; depth::MAX_PLY_SLOTS]),
            pvs: Box::new([PrincipalVariation::EMPTY; depth::MAX_PLY_SLOTS]),
            killers: Box::new([[MaybePos::NONE; 2]; depth::MAX_PLY_SLOTS]),
            lmr_table: Box::new(build_lmr_table(config)),
            debug_statics: Box::new([DebugStatics::EMPTY; depth::MAX_PLY_SLOTS]),
            root_pv: PrincipalVariation::EMPTY,
            root_moves_in_1k: [0; pos::BOARD_SIZE],
            singular_root: false,
            batch_counter: BatchCounter::new(global_counter_in_1k),
            aborted,
            best_move: MaybePos::NONE,
            selective_depth: Depth::ZERO,
            ply: 0,
        }
    }

    pub fn should_check_limit(&self) -> bool {
        self.batch_counter.buffer_zero()
    }

    pub fn search_limit_exceeded(&self) -> bool {
        self.thread_type.time_manager().is_hard_limit_reached()
            || self.config.max_nodes_in_1k.is_some_and(|in_1k|
                self.batch_counter.count_global_in_1k() >= in_1k
            )
    }

    pub fn set_aborted(&self) {
        self.aborted.store(true, Ordering::Relaxed);
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::Relaxed)
    }

    pub fn lookup_lmr_table(&self, depth_left: Depth, moves_made: usize) -> Depth {
        let depth_clamped = depth_left.clamp_value(Depth::from_i32(63)).value();
        let moves_made_clamped = moves_made.clamp(0, depth::MAX_PLY);

        self.lmr_table[depth_clamped as usize][moves_made_clamped]
    }

    pub fn push_ply(&mut self, pos: Pos) {
        self.ply += 1;
        self.ss[self.ply].pos = pos.into();
    }

    pub fn pop_ply(&mut self) {
        self.ply -= 1;
    }

    pub fn push_killer(&mut self, pos: Pos) {
        if self.killers[self.ply].contains(&pos.into()) {
            return;
        }

        self.killers[self.ply][1] = self.killers[self.ply][0];
        self.killers[self.ply][0] = pos.into();
    }

    pub fn clear_killer(&mut self) {
        if self.ply + 2 < depth::MAX_PLY {
            self.killers[self.ply + 2] = [MaybePos::NONE; 2];
        }
    }

}

fn build_lmr_table(config: Config) -> [[Depth; depth::MAX_PLY_SLOTS]; 64] {
    let mut lmr_table = [[Depth::ZERO; depth::MAX_PLY_SLOTS]; 64];

    let worker_factor = 1.0 + (config.workers.min(16) as f64) / 100.0;
    let lmr_div = params::LMR_DIV * worker_factor;

    for depth in 1 .. 64 {
        for played in 1 .. depth::MAX_PLY_SLOTS {
            lmr_table[depth][played] = Depth::from_i32(
                (params::LMR_BASE + (depth as f64).ln() * (played as f64).ln() / lmr_div) as i32
            );
        }
    }

    lmr_table
}
