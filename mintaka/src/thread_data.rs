use crate::batch_counter::BatchCounter;
use crate::config::Config;
use crate::endgame::vcf::VCFFrame;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::search_limit::{NodeCount, SearchLimit, TimeBound};
use rusty_renju::notation::value::Depth;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

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

    start_time: Instant,
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
            start_time: Instant::now(),
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
            SearchLimit::Time(TimeBound { duration }) =>
                self.start_time.elapsed() >= duration,
            SearchLimit::Infinite => false
        }
    }

    pub fn calculate_tps(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let nodes = self.batch_counter.count_global() as f64;
        nodes / elapsed
    }

}
