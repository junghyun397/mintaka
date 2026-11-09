use crate::batch_counter::BatchCounter;
use crate::config::{Config, SearchObjective};
use crate::eval::evaluator::ActiveEvaluator;
use crate::game_state::RecoveryState;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TTView;
use crate::params;
use crate::principal_variation::PrincipalVariation;
use crate::search::SearchResult;
use crate::thread_type::ThreadType;
use crate::utils::depth;
use crate::utils::depth::Depth;
use crate::utils::score_kind::ScoreKind;
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::score::{MaybeScore, Score};
use std::cmp::Reverse;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub const KILLER_MOVE_SLOTS: usize = 2;

#[derive(Copy, Clone)]
pub struct RootMove {
    pub pos: Pos,
    pub nodes: u64,
    pub score: Score,
    pub score_kind: ScoreKind,
    pub pv: PrincipalVariation,
    pub depth: Depth,
    pub selective_depth: Depth,
}

impl RootMove {
    pub fn new(pos: Pos) -> Self {
        let mut pv = PrincipalVariation::EMPTY;
        pv.set(pos);

        Self {
            pos,
            nodes: 0,
            score: Score::NEG_INF,
            score_kind: ScoreKind::UpperBound,
            pv,
            depth: Depth::ZERO,
            selective_depth: Depth::ZERO,
        }
    }

    pub fn update(
        &mut self,
        score: Score,
        child_pv: &PrincipalVariation,
        depth: Depth,
        selective_depth: Depth,
        alpha: Score,
        beta: Score,
    ) {
        self.score = score;

        self.score_kind = ScoreKind::from_alpha_beta(score, alpha, beta);

        self.pv.update(self.pos, child_pv);
        self.depth = depth;
        self.selective_depth = selective_depth;
    }
}

impl From<&RootMove> for SearchResult {
    fn from(root_move: &RootMove) -> Self {
        SearchResult {
            score: root_move.score,
            pv: root_move.pv,
            depth: root_move.depth,
            selective_depth: root_move.selective_depth,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SearchFrame {
    pub pos: MaybePos,
    pub evaluator_eval: MaybeScore,
    pub static_eval: Score,
    pub on_pv: bool,
    pub searching: MaybePos,
}

impl SearchFrame {
    pub const EMPTY: Self = Self {
        pos: MaybePos::NONE,
        evaluator_eval: MaybeScore::NONE,
        static_eval: Score::DRAW,
        on_pv: false,
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
pub struct ThreadData<'a, const R: RuleKind, TH: ThreadType> {
    pub thread_type: TH,
    pub search_objective: SearchObjective,
    pub tid: u32,
    pub config: Config,

    pub evaluator: ActiveEvaluator<R>,

    pub tt: TTView<'a>,
    pub ht: Box<HistoryTable>,
    pub ss: Box<[SearchFrame; depth::MAX_PLY_SLOTS]>,
    pub recovery_states: Box<[RecoveryState; depth::MAX_PLY_SLOTS]>,
    pub killers: Box<[[MaybePos; KILLER_MOVE_SLOTS]; depth::MAX_PLY_SLOTS]>,
    pub debug_statics: Box<[DebugStatics; depth::MAX_PLY_SLOTS]>,

    pub lmr_table: Box<[[Depth; depth::MAX_PLY_SLOTS]; 64]>,

    pub root_moves: Vec<RootMove>,
    pub root_indexes: [u8; pos::BOARD_SIZE],
    pub singular_root: bool,

    pub batch_counter: BatchCounter<'a>,
    aborted: &'a AtomicBool,

    pub selective_depth: Depth,

    pub ply: usize,
}

impl<'a, const R: RuleKind, TH: ThreadType> ThreadData<'a, R, TH> {
    pub fn new(
        thread_type: TH,
        tid: u32,
        search_objective: SearchObjective,
        config: Config,
        evaluator: ActiveEvaluator<R>,
        tt: TTView<'a>,
        ht: HistoryTable,
        aborted: &'a AtomicBool,
        global_counter_in_1k: &'a AtomicU32,
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
            recovery_states: Box::new([RecoveryState::EMPTY; depth::MAX_PLY_SLOTS]),
            killers: Box::new([[MaybePos::NONE; 2]; depth::MAX_PLY_SLOTS]),
            lmr_table: Box::new(build_lmr_table(config)),
            debug_statics: Box::new([DebugStatics::EMPTY; depth::MAX_PLY_SLOTS]),
            root_moves: Vec::with_capacity(pos::BOARD_SIZE),
            root_indexes: [u8::MAX; pos::BOARD_SIZE],
            singular_root: false,
            batch_counter: BatchCounter::new(global_counter_in_1k),
            aborted,
            selective_depth: Depth::ZERO,
            ply: 0,
        }
    }

    pub fn should_check_limit(&self) -> bool {
        self.batch_counter.buffer_zero()
    }

    pub fn clear_root_moves(&mut self) {
        self.root_moves.clear();
        self.root_indexes.fill(u8::MAX);
    }

    pub fn ensure_root_move(&mut self, pos: Pos) {
        if self.root_indexes[pos.idx_usize()] == u8::MAX {
            self.root_indexes[pos.idx_usize()] = self.root_moves.len() as u8;
            self.root_moves.push(RootMove::new(pos));
        }
    }

    pub fn sort_root_moves(&mut self) {
        self.root_moves.sort_by_key(|root_move| Reverse((
            root_move.score_kind != ScoreKind::UpperBound,
            root_move.depth,
            root_move.score,
        )));

        for (index, root_move) in self.root_moves.iter().enumerate() {
            self.root_indexes[root_move.pos.idx_usize()] = index as u8;
        }
    }

    pub fn search_limit_exceeded(&self) -> bool {
        self.thread_type
            .time_manager()
            .is_hard_limit_reached(&self.batch_counter)
    }

    pub fn set_aborted(&self) {
        self.aborted.store(true, Ordering::Relaxed);
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::Relaxed)
    }

    pub fn lookup_lmr_table(&self, depth_left: Depth, moves_made: usize) -> Depth {
        let depth_clamped = depth_left.clamp_value(Depth::from_i32(63)).value_i32();
        let moves_made_clamped = moves_made.clamp(0, depth::MAX_PLY);

        self.lmr_table[depth_clamped as usize][moves_made_clamped]
    }

    pub fn push_ply(&mut self, pos: Pos, recovery_state: RecoveryState) {
        self.recovery_states[self.ply] = recovery_state;
        self.ply += 1;
        self.ss[self.ply].pos = pos.into();
    }

    pub fn pop_ply(&mut self) -> &RecoveryState {
        self.ply -= 1;
        &self.recovery_states[self.ply]
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

    for depth in 1..64 {
        for played in 1..depth::MAX_PLY_SLOTS {
            lmr_table[depth][played] = Depth::from_i32(
                (params::LMR_BASE + (depth as f64).ln() * (played as f64).ln() / lmr_div) as i32,
            );
        }
    }

    lmr_table
}
