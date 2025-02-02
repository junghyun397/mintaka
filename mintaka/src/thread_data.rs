use rusty_renju::history::History;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct ThreadData<'a> {
    pub aborted: &'a AtomicBool,
    pub history_table: History
}

impl<'a> ThreadData<'a> {

    fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::Relaxed)
    }

}
