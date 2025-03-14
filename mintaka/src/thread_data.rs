use crate::batch_counter::BatchCounter;
use crate::config::Config;
use crate::endgame::vcf::VcfFrame;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TTView;
use crate::search_frame::SearchFrame;
use crate::thread_type::ThreadType;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

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
    global_aborted: &'a AtomicBool,
}

impl<'a, TH: ThreadType> ThreadData<'a, TH> {

    pub fn new(
        thread_type: TH, tid: usize,
        config: Config,
        tt: TTView<'a>,
        ht: HistoryTable,
        global_aborted: &'a AtomicBool,
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
            global_aborted,
        }
    }

    pub fn is_aborted(&self) -> bool {
        self.global_aborted.load(Ordering::Relaxed)
    }

    pub fn calculate_tps(&self, elapsed: Duration) -> f64 {
        let elapsed = elapsed.as_secs_f64();
        let nodes = self.batch_counter.count_global() as f64;
        nodes / elapsed
    }

}
