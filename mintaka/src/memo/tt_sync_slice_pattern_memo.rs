use crate::memo::abstract_transposition_table::{AbstractTTEntry, AbstractTranspositionTable};
use crate::memo::hash_key::HashKey;
use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::slice_pattern::SlicePattern;

pub struct TTSyncSlicePatternEntry(u64, SlicePattern);

impl AbstractTTEntry for TTSyncSlicePatternEntry {

    fn clear_mut(&mut self) {
        self.0 = 0;
        self.1 = SlicePattern::EMPTY;
    }

    fn usage(&self) -> usize {
        self.0.min(2) as usize
    }

}

impl TTSyncSlicePatternEntry {

    fn is_hit(&self, packed_slice: u64) -> bool {
        self.0 == packed_slice
    }

    // interior-mutability
    fn store_mut(&mut self, packed_slice: u64, pattern: &SlicePattern) {
        self.0 = packed_slice;
        self.1 = *pattern;
    }

    fn decode(&self) -> SlicePattern {
        self.1
    }

}

pub struct TTSyncSlicePatternMemo {
    table: Vec<TTSyncSlicePatternEntry>,
}

impl Default for TTSyncSlicePatternMemo {

    fn default() -> Self {
        let mut new = Self {
            table: Vec::new(),
        };

        new.resize_mut(32);

        new
    }

}

impl AbstractTranspositionTable<TTSyncSlicePatternEntry> for TTSyncSlicePatternMemo {

    fn internal_table(&self) -> &Vec<TTSyncSlicePatternEntry> {
        &self.table
    }

    fn internal_table_mut(&mut self) -> &mut Vec<TTSyncSlicePatternEntry> {
        &mut self.table
    }

    fn assign_internal_table_mut(&mut self, table: Vec<TTSyncSlicePatternEntry>) {
        self.table = table;
    }

}

impl SlicePatternMemo for TTSyncSlicePatternMemo {

    fn prefetch_memo(&self, packed_slice: u64) {
        self.prefetch(Self::build_hash_key(packed_slice));
    }

    fn probe_or_put_mut<F>(&mut self, packed_slice: u64, produce: F) -> SlicePattern
    where F: FnOnce() -> SlicePattern
    {
        let atomic_entry = {
            let slice_hash = Self::build_hash_key(packed_slice);
            let idx = self.calculate_index(slice_hash);
            &mut self.table[idx]
        };

        if atomic_entry.is_hit(packed_slice) {
            return atomic_entry.decode();
        }

        let slice_pattern = produce();

        atomic_entry.store_mut(
            packed_slice,
            &slice_pattern
        );

        slice_pattern
    }

}

impl TTSyncSlicePatternMemo {

    fn build_hash_key(packed_slice: u64) -> HashKey {
        // fibonacci-hashing
        HashKey(packed_slice.wrapping_mul(0x9e3779b97f4a7c15))
    }

}
