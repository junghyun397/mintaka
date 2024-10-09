use crate::memo::hash_key::HashKey;
use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::slice_pattern::SlicePattern;
use crate::utils::abstract_transposition_table::AbstractTranspositionTable;
use std::sync::atomic::{AtomicU64, Ordering};

struct TTSlicePatternEntry {
    concat_slice: u32,
    data: SlicePattern,
}

struct AtomicTTSlicePatternEntry(AtomicU64, AtomicU64, AtomicU64, AtomicU64, AtomicU64);

pub struct TTSlicePatternMemo {
    table: Vec<AtomicTTSlicePatternEntry>,
}

impl Default for TTSlicePatternMemo {

    fn default() -> Self {
        const SIZE_IN_MIB: usize = 256; // 1 MiB = 30,000 slice patterns
        const SIZE: usize = SIZE_IN_MIB * 1024 * 1024 / size_of::<SlicePattern>();
        todo!()
    }

}

impl AbstractTranspositionTable<AtomicTTSlicePatternEntry> for TTSlicePatternMemo {

    fn internal_table(&self) -> &Vec<AtomicTTSlicePatternEntry> {
        &self.table
    }

    fn assign_internal_table_mut(&mut self, table: Vec<AtomicTTSlicePatternEntry>) {
        self.table = table;
    }

}

impl SlicePatternMemo for TTSlicePatternMemo {

    fn probe_or_put_mut<F>(&mut self, key: HashKey, produce: F) -> &SlicePattern
    where F: FnOnce() -> SlicePattern
    {
        let idx = self.calculate_index(key);
        let atomic_entry = &self.table[idx];

        if atomic_entry.0.load(Ordering::Relaxed) & 0x00000000_FFFFFFFF == key.0 & 0x00000000_FFFFFFFF {
            return todo!()
        }

        todo!()
    }

}
