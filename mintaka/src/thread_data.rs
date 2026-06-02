use crate::batch_counter::BatchCounter;
use crate::config::{Config, SearchObjective};
use crate::eval::evaluator::Evaluator;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TTView;
use crate::principal_variation::PrincipalVariation;
use crate::search_endgame::EndgameFrame;
use crate::game_state::RecoveryState;
use crate::thread_type::ThreadType;
use crate::value::Depth;
use crate::{params, value};
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::score::{Score, Scores};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use rusty_renju::notation::rule::RuleKind;

pub const KILLER_MOVE_SLOTS: usize = 2;

#[derive(Debug, Copy, Clone)]
pub struct SearchFrame {
    pub pos: MaybePos,
    pub evaluator_eval: Score,
    pub static_eval: Score,
    pub on_pv: bool,
    pub recovery_state: RecoveryState,
    pub searching: MaybePos,
}

impl SearchFrame {
    pub const EMPTY: Self = Self {
        pos: MaybePos::NONE,
        evaluator_eval: Score::DRAW,
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
    pub ss: Box<[SearchFrame; value::MAX_PLY_SLOTS]>,
    pub pvs: Box<[PrincipalVariation; value::MAX_PLY_SLOTS]>,
    pub killers: Box<[[MaybePos; KILLER_MOVE_SLOTS]; value::MAX_PLY_SLOTS]>,
    pub debug_statics: Box<[DebugStatics; value::MAX_PLY_SLOTS]>,

    pub lmr_table: Box<[[Depth; value::MAX_PLY_SLOTS]; 64]>,

    pub root_pv: PrincipalVariation,
    pub singular_root: bool,

    pub endgame_stack: Box<[EndgameFrame; value::MAX_PLY_SLOTS]>,
    pub endgame_stack_top: usize,

    pub batch_counter: BatchCounter<'a>,
    aborted: &'a AtomicBool,

    pub best_move: MaybePos,
    pub selective_depth: usize,

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
            ss: Box::new([SearchFrame::EMPTY; value::MAX_PLY_SLOTS]),
            pvs: Box::new([PrincipalVariation::EMPTY; value::MAX_PLY_SLOTS]),
            killers: Box::new([[MaybePos::NONE; 2]; value::MAX_PLY_SLOTS]),
            lmr_table: Box::new(build_lmr_table(config)),
            debug_statics: Box::new([DebugStatics::EMPTY; value::MAX_PLY_SLOTS]),
            root_pv: PrincipalVariation::EMPTY,
            singular_root: false,
            endgame_stack: Box::new([EndgameFrame::EMPTY; value::MAX_PLY_SLOTS]),
            endgame_stack_top: 0,
            batch_counter: BatchCounter::new(global_counter_in_1k),
            aborted,
            best_move: MaybePos::NONE,
            selective_depth: 0,
            ply: 0,
        }
    }

    pub fn should_check_limit(&self) -> bool {
        self.batch_counter.buffer_zero()
    }

    pub fn search_limit_exceeded(&self) -> bool {
        self.thread_type.time_exceeded()
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
        let depth_clamped = depth_left.clamp(0, 63) as usize;
        let moves_made_clamped = moves_made.clamp(0, value::MAX_PLY);

        self.lmr_table[depth_clamped][moves_made_clamped]
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
        if self.ply + 2 < value::MAX_PLY {
            self.killers[self.ply + 2] = [MaybePos::NONE; 2];
        }
    }

    pub fn clear_endgame_stack(&mut self) {
        self.endgame_stack_top = 0;
    }

    pub fn push_endgame_frame(&mut self, frame: EndgameFrame) {
        self.endgame_stack[self.endgame_stack_top] = frame;
        self.endgame_stack_top += 1;
    }

    pub fn pop_endgame_frame(&mut self) -> Option<EndgameFrame> {
        if self.endgame_stack_top == 0 {
            return None;
        }

        self.endgame_stack_top -= 1;
        Some(self.endgame_stack[self.endgame_stack_top])
    }
}

fn build_lmr_table(config: Config) -> [[Depth; value::MAX_PLY_SLOTS]; 64] {
    let mut lmr_table = [[0; value::MAX_PLY_SLOTS]; 64];

    let worker_factor = 1.0 + (config.workers.min(16) as f64) / 100.0;
    let lmr_div = params::LMR_DIV * worker_factor;

    for depth in 1 .. 64 {
        for played in 1 .. value::MAX_PLY_SLOTS {
            lmr_table[depth][played] = (
                params::LMR_BASE +
                    (depth as f64).ln() * (played as f64).ln() / lmr_div
            ) as Depth;
        }
    }

    lmr_table
}
