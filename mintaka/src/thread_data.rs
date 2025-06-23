use crate::batch_counter::BatchCounter;
use crate::config::Config;
use crate::endgame::vcf_search::VcfFrame;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TTView;
use crate::parameters::MAX_PLY;
use crate::principal_variation::PrincipalVariation;
use crate::search_frame::{SearchFrame, KILLER_MOVE_SLOTS};
use crate::thread_type::ThreadType;
use rusty_renju::notation::pos::{MaybePos, Pos};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Clone)]
pub struct ThreadData<'a, TH: ThreadType> {
    pub thread_type: TH,
    pub tid: usize,
    pub config: Config,

    pub tt: TTView<'a>,
    pub ht: Box<HistoryTable>,
    pub ss: Box<[SearchFrame; MAX_PLY + 1]>,
    pub pvs: Box<[PrincipalVariation; MAX_PLY + 1]>,
    pub killers: Box<[[MaybePos; KILLER_MOVE_SLOTS]; MAX_PLY + 1]>,

    pub vcf_stack: Box<[VcfFrame; MAX_PLY + 1]>,
    pub vcf_stack_top: usize,

    pub batch_counter: BatchCounter<'a>,
    aborted: &'a AtomicBool,

    pub best_move: MaybePos,
    pub depth: usize,
    pub ply: usize,
}

impl<'a, TH: ThreadType> ThreadData<'a, TH> {

    pub fn new(
        thread_type: TH, tid: usize,
        config: Config,
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
            ht: Box::new(ht),
            ss: Box::new(unsafe { std::mem::MaybeUninit::uninit().assume_init() }),
            pvs: Box::new(unsafe { std::mem::MaybeUninit::uninit().assume_init() }),
            killers: Box::new(unsafe { std::mem::MaybeUninit::uninit().assume_init() }),
            vcf_stack: Box::new(unsafe { std::mem::MaybeUninit::uninit().assume_init() }),
            vcf_stack_top: 0,
            batch_counter: BatchCounter::new(global_counter_in_1k),
            aborted,
            best_move: MaybePos::NONE,
            depth: 0,
            ply: 0,
        }
    }

    pub fn should_check_limit(&self) -> bool {
        self.batch_counter.count_local_total() % 1023 == 0
    }

    pub fn search_limit_exceeded(&self) -> bool {
        self.thread_type.limit_exceeded(self.batch_counter.count_global_in_1k())
    }

    pub fn set_aborted_mut(&self) {
        self.aborted.store(true, Ordering::Relaxed);
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::Relaxed)
    }

    pub fn push_ply_mut(&mut self) {
        self.ply += 1;
        self.batch_counter.increment_single_mut();
    }

    pub fn pop_ply_mut(&mut self) {
        self.ply -= 1;
    }

    pub fn insert_killer_move_mut(&mut self, pos: Pos) {
        self.killers[self.ply][1] = self.killers[self.ply][0];
        self.killers[self.ply][0] = pos.into();
    }

    pub fn clear_killer_move_mut(&mut self) {
        self.killers[self.ply] = [MaybePos::NONE; 2];
    }

    pub fn update_history_table_mut(&mut self, pos: Pos) {
        // TODO
    }

    pub fn clear_vcf_stack_mut(&mut self) {
        self.vcf_stack_top = 0;
    }

    pub fn push_vcf_frame_mut(&mut self, frame: VcfFrame) {
        self.vcf_stack[self.vcf_stack_top] = frame;
        self.vcf_stack_top += 1;
    }

    pub fn pop_vcf_frame_mut(&mut self) -> Option<VcfFrame> {
        if self.vcf_stack_top == 0 {
            return None;
        }

        self.vcf_stack_top -= 1;
        Some(self.vcf_stack[self.vcf_stack_top])
    }

}
