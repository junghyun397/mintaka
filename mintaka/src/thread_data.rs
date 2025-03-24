use crate::batch_counter::BatchCounter;
use crate::config::Config;
use crate::endgame::vcf::VcfFrame;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TTView;
use crate::search_frame::SearchFrame;
use crate::thread_type::ThreadType;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Clone)]
pub struct ThreadData<'a, TH: ThreadType> {
    pub thread_type: TH,
    pub tid: usize,
    pub config: Config,

    pub tt: TTView<'a>,
    pub ht: HistoryTable,
    pub search_stack: [SearchFrame; 128],
    pub vcf_stack: Vec<VcfFrame>,

    pub batch_counter: BatchCounter<'a>,
    aborted: &'a AtomicBool,
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
            search_stack: [SearchFrame::default(); 128],
            vcf_stack: Vec::with_capacity(32),
            batch_counter: BatchCounter::new(global_counter_in_1k),
            aborted,
        }
    }

    pub fn search_limit_exceeded(&self) -> bool {
        self.thread_type.time_limit_exceeded()
            || self.batch_counter.count_global() >= self.config.max_nodes_in_1k
    }

    pub fn node_limit_exceeded(&self) -> bool {
        self.batch_counter.count_global() >= self.config.max_nodes_in_1k
    }

    pub fn set_aborted_mut(&self) {
        self.aborted.store(true, Ordering::Relaxed);
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::Relaxed)
    }

}
