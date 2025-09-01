use crate::batch_counter::BatchCounter;
use crate::config::Config;
use crate::endgame::accumulator::EndgameFrame;
use crate::eval::evaluator::Evaluator;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TTView;
use crate::principal_variation::PrincipalVariation;
use crate::search_frame::{SearchFrame, KILLER_MOVE_SLOTS};
use crate::thread_type::ThreadType;
use crate::value::{Depth, MAX_PLY};
use rusty_renju::notation::pos;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::Score;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Clone)]
pub struct ThreadData<'a, TH: ThreadType, E: Evaluator> {
    pub thread_type: TH,
    pub tid: u32,
    pub config: Config,

    pub evaluator: E,

    pub tt: TTView<'a>,
    pub ht: Box<HistoryTable>,
    pub ss: Box<[SearchFrame; MAX_PLY]>,
    pub pvs: Box<[PrincipalVariation; MAX_PLY]>,
    pub killers: Box<[[MaybePos; KILLER_MOVE_SLOTS]; MAX_PLY]>,

    pub vcf_stack: Box<[EndgameFrame; MAX_PLY]>,
    pub endgame_stack_top: usize,

    pub batch_counter: BatchCounter<'a>,
    aborted: &'a AtomicBool,

    pub best_move: MaybePos,
    pub depth: Depth,
    pub depth_reached: Depth,
    pub ply: usize,

    pub root_scores: [f32; pos::BOARD_SIZE],
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
            killers: Box::new([[MaybePos::NONE; 2]; MAX_PLY]),
            vcf_stack: Box::new(unsafe { std::mem::MaybeUninit::uninit().assume_init() }),
            endgame_stack_top: 0,
            batch_counter: BatchCounter::new(global_counter_in_1k),
            aborted,
            best_move: MaybePos::NONE,
            depth: 0,
            depth_reached: 0,
            ply: 0,
            root_scores: [f32::NAN; pos::BOARD_SIZE],
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

    pub fn set_aborted_mut(&self) {
        self.aborted.store(true, Ordering::Relaxed);
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::Relaxed)
    }

    pub fn push_root_move(&mut self, pos: Pos, score: Score) {
        self.root_scores[pos.idx_usize()] = score as f32;
    }

    pub fn push_ply_mut(&mut self, pos: Pos) {
        self.ply += 1;
        self.batch_counter.increment_single_mut();

        self.ss[self.ply].last_pos = pos.into();
    }

    pub fn pop_ply_mut(&mut self) {
        self.ply -= 1;
    }

    pub fn push_killer_move_mut(&mut self, pos: Pos) {
        self.killers[self.ply][1] = self.killers[self.ply][0];
        self.killers[self.ply][0] = pos.into();
    }

    pub fn clear_killer_move_mut(&mut self) {
        self.killers[self.ply] = [MaybePos::NONE; 2];
    }

    pub fn update_history_table_mut(&mut self, pos: Pos) {
        // TODO
    }

    pub fn clear_endgame_stack_mut(&mut self) {
        self.endgame_stack_top = 0;
    }

    pub fn push_endgame_frame_mut(&mut self, frame: EndgameFrame) {
        self.vcf_stack[self.endgame_stack_top] = frame;
        self.endgame_stack_top += 1;
    }

    pub fn pop_endgame_frame_mut(&mut self) -> Option<EndgameFrame> {
        if self.endgame_stack_top == 0 {
            return None;
        }

        self.endgame_stack_top -= 1;
        Some(self.vcf_stack[self.endgame_stack_top])
    }

}
