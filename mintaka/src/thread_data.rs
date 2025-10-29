use crate::batch_counter::BatchCounter;
use crate::config::Config;
use crate::eval::evaluator::Evaluator;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TTView;
use crate::movegen::move_picker;
use crate::principal_variation::PrincipalVariation;
use crate::search_endgame::EndgameFrame;
use crate::search_frame::SearchFrame;
use crate::thread_type::ThreadType;
use crate::value;
use crate::value::{Depth, MAX_PLY};
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::score::Score;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RootMove {
    pub score: RootScore,
    pub nodes_in_1k: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RootScore {
    Exact(Score),
    FailHigh,
    FailLow,
    #[default] Unknown,
}

#[derive(Clone)]
pub struct ThreadData<'a, TH: ThreadType, E: Evaluator> {
    pub thread_type: TH,
    pub tid: u32,
    pub config: Config,

    pub evaluator: E,

    pub tt: TTView<'a>,
    pub ht: Box<HistoryTable>,
    pub ss: Box<[SearchFrame; value::MAX_PLY_SLOTS]>,
    pub pvs: Box<[PrincipalVariation; value::MAX_PLY_SLOTS]>,
    pub killers: Box<[[MaybePos; move_picker::KILLER_MOVE_SLOTS]; value::MAX_PLY_SLOTS]>,

    pub lmr_table: Box<[[Depth; value::MAX_PLY_SLOTS]; 64]>,

    pub root_pv: PrincipalVariation,
    pub singular_root: bool,

    pub vcf_stack: Box<[EndgameFrame; MAX_PLY + 1]>,
    pub endgame_stack_top: usize,

    pub batch_counter: BatchCounter<'a>,
    aborted: &'a AtomicBool,

    pub best_move: MaybePos,
    pub selective_depth: usize,

    pub ply: usize,
}

impl<'a, TH: ThreadType, E: Evaluator> ThreadData<'a, TH, E> {

    #[allow(clippy::uninit_assumed_init)]
    pub fn new(
        thread_type: TH, tid: u32,
        config: Config,
        evaluator: E,
        tt: TTView<'a>,
        ht: HistoryTable,
        aborted: &'a AtomicBool,
        global_counter_in_1k: &'a AtomicUsize
    ) -> Self {
        Self {
            thread_type,
            tid,
            config,
            tt,
            evaluator,
            ht: Box::new(ht),
            ss: Box::new(unsafe { std::mem::MaybeUninit::uninit().assume_init() }),
            pvs: Box::new(unsafe { std::mem::MaybeUninit::uninit().assume_init() }),
            killers: Box::new([[MaybePos::NONE; 2]; value::MAX_PLY_SLOTS]),
            lmr_table: Box::new(build_lmr_table(config)),
            root_pv: PrincipalVariation::EMPTY,
            singular_root: false,
            vcf_stack: Box::new(unsafe { std::mem::MaybeUninit::uninit().assume_init() }),
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
        let moves_made_clamped = moves_made.clamp(0, 63);

        self.lmr_table[depth_clamped][moves_made_clamped]
    }

    pub fn push_ply(&mut self, pos: Pos) {
        self.ply += 1;
        self.batch_counter.increment_single();
        self.ss[self.ply].pos = pos.into();
        self.ss[self.ply].cutoffs = 0;
    }

    pub fn pop_ply(&mut self) {
        self.ply -= 1;
    }

    pub fn push_killer(&mut self, pos: Pos) {
        self.killers[self.ply][1] = self.killers[self.ply][0];
        self.killers[self.ply][0] = pos.into();
    }

    pub fn clear_killer(&mut self) {
        self.killers[self.ply + 1] = [MaybePos::NONE; 2];
    }

    pub fn clear_endgame_stack(&mut self) {
        self.endgame_stack_top = 0;
    }

    pub fn push_endgame_frame(&mut self, frame: EndgameFrame) {
        self.vcf_stack[self.endgame_stack_top] = frame;
        self.endgame_stack_top += 1;
    }

    pub fn pop_endgame_frame(&mut self) -> Option<EndgameFrame> {
        if self.endgame_stack_top == 0 {
            return None;
        }

        self.endgame_stack_top -= 1;
        Some(self.vcf_stack[self.endgame_stack_top])
    }

}

fn build_lmr_table(config: Config) -> [[Depth; value::MAX_PLY_SLOTS]; 64] {
    let mut lmr_table = [[0; value::MAX_PLY_SLOTS]; 64];

    let worker_factor = 1.0 + (config.workers.min(16) as f64) / 100.0;
    let lmr_div = value::LMR_DIV * worker_factor;

    for depth in 0 .. 64 {
        for played in 0 .. 64 {
            lmr_table[depth][played] = (value::LMR_BASE + (depth as f64).ln() * (played as f64).ln() / lmr_div) as Depth;
        }
    }

    lmr_table
}
