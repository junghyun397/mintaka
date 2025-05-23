use crate::batch_counter::BatchCounter;
use crate::config::Config;
use crate::endgame::vcf_search::VcfFrame;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TTView;
use crate::movegen::movegen_window::MovegenWindow;
use crate::parameters::MAX_PLY;
use crate::principal_variation::PrincipalVariation;
use crate::search_frame::{SearchFrame, KILLER_MOVE_SLOTS};
use crate::thread_type::ThreadType;
use arrayvec::ArrayVec;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::value::Depth;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Clone)]
pub struct ThreadData<'a, TH: ThreadType> {
    pub thread_type: TH,
    pub tid: usize,
    pub config: Config,

    pub tt: TTView<'a>,
    pub ht: HistoryTable,
    pub ss: ArrayVec<SearchFrame, MAX_PLY>,
    pub pvs: ArrayVec<PrincipalVariation, MAX_PLY>,
    pub killers: ArrayVec<[MaybePos; KILLER_MOVE_SLOTS], MAX_PLY>,
    pub counters: ArrayVec<MaybePos, MAX_PLY>,

    pub movegen_stack: ArrayVec<MovegenWindow, MAX_PLY>,
    pub vcf_stack: Vec<VcfFrame>,

    pub batch_counter: BatchCounter<'a>,
    aborted: &'a AtomicBool,

    pub best_move: MaybePos,
    pub depth: Depth,
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
            tt, ht,
            ss: ArrayVec::new_const(),
            pvs: ArrayVec::new_const(),
            killers: ArrayVec::new_const(),
            counters: ArrayVec::new_const(),
            movegen_stack: ArrayVec::new_const(),
            vcf_stack: Vec::with_capacity(32),
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

    pub fn push_ply_mut(
        &mut self,
        movegen_window: MovegenWindow,
    ) {
        self.ply += 1;
        self.batch_counter.add_single_mut();
        self.movegen_stack[self.ply] = movegen_window;
    }

    pub fn pop_ply_mut(&mut self) -> MovegenWindow {
        self.ply -= 1;
        self.movegen_stack[self.ply]
    }

    pub fn insert_killer_move_mut(&mut self, pos: Pos) {
        if self.killers[self.ply][0].is_none() {
            self.killers[self.ply][0] = pos.into();
        } else {
            self.killers[self.ply][1] = pos.into();
        }
    }

}
