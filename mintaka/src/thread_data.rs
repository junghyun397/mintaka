use crate::endgame::vcf::VCFFrame;
use crate::memo::transposition_table::TranspositionTable;
use crate::utils::batch_counter::BatchCounter;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct ThreadData<'a> {
    pub tt: &'a TranspositionTable,
    pub vcf_stack: Vec<VCFFrame>,
    pub batch_counter: BatchCounter<'a>,
    global_aborted: &'a AtomicBool,
}

impl<'a> ThreadData<'a> {

    pub fn new(tt: &'a TranspositionTable, global_aborted: &'a AtomicBool, global_counter: &'a AtomicUsize) -> Self {
        Self {
            tt,
            vcf_stack: Vec::with_capacity(32),
            batch_counter: BatchCounter::new(global_counter),
            global_aborted,
        }
    }

    fn is_aborted(&self) -> bool {
        self.global_aborted.load(Ordering::Relaxed)
    }

}
