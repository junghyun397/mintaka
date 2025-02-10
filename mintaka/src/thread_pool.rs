use crate::config::Config;
use crate::memo::transposition_table::TranspositionTable;
use crate::protocol::game_manager::GameManager;
use crate::thread_data::ThreadData;
use std::sync::atomic::{AtomicBool, AtomicUsize};

pub struct ThreadPool<'a> {
    pub workers: Vec<ThreadData<'a>>,
}

impl<'a> ThreadPool<'a> {

    pub fn new(
        manager: &'a dyn GameManager, config: Config,
        tt: &'a TranspositionTable, global_aborted: &'a AtomicBool, global_counter_in_1k: &'a AtomicUsize,
        threads: usize
    ) -> Self {
        Self {
            workers: (0..threads)
                .map(|_| ThreadData::new(manager, config, tt, global_aborted, global_counter_in_1k))
                .collect(),
        }
    }

}
