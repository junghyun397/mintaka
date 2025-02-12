use crate::batch_counter::BatchCounter;
use crate::config::Config;
use crate::endgame::vcf::VCFFrame;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::search_limit::{NodeCount, SearchLimit, TimeBound};
use rusty_renju::notation::value::Depth;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Clone)]
pub struct ThreadData<'a> {
    pub tid: usize,
    pub config: Config,
    pub search_limit: SearchLimit,

    pub tt: &'a TranspositionTable,
    pub ht: HistoryTable,
    pub vcf_stack: Vec<VCFFrame>,

    pub batch_counter: BatchCounter<'a>,
    global_aborted: &'a AtomicBool,
}

impl<'a> ThreadData<'a> {

    pub fn new(
        tid: usize, config: Config, search_limit: SearchLimit,
        tt: &'a TranspositionTable, ht: HistoryTable,
        global_aborted: &'a AtomicBool, global_counter: &'a AtomicUsize
    ) -> Self {
        Self {
            tid, config, search_limit,
            tt, ht,
            vcf_stack: Vec::with_capacity(32),
            batch_counter: BatchCounter::new(global_counter),
            global_aborted,
        }
    }

    pub fn is_aborted(&self) -> bool {
        self.global_aborted.load(Ordering::Relaxed)
    }

    pub fn limit_reached(&self, current_depth: Depth) -> bool {
        match self.search_limit {
            SearchLimit::Depth(depth) =>
                current_depth >= depth,
            SearchLimit::Nodes(NodeCount { nodes_in_1k }) =>
                self.batch_counter.count_global() >= nodes_in_1k,
            SearchLimit::Time(TimeBound { epoch_time_lower_64 }) =>
                32 >= epoch_time_lower_64,
            SearchLimit::Infinite => false
        }
    }

}
