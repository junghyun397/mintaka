use crate::memo::abstract_transposition_table::{AbstractTTEntry, AbstractTranspositionTable};
use crate::memo::hash_key::HashKey;
use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::slice_pattern::SlicePattern;
use std::sync::atomic::{AtomicU64, Ordering};

// layout: block 0 = slice key 32 bits
//         block 1, 2 = black_patterns 128 bits
//         block 3, 4 = white_patterns 128 bits
pub struct AtomicTTSlicePatternEntry(AtomicU64, AtomicU64, AtomicU64);

impl AbstractTTEntry for AtomicTTSlicePatternEntry {

    fn clear_mut(&mut self) {
        self.0.store(0, Ordering::Relaxed);
        self.1.store(0, Ordering::Relaxed);
        self.2.store(0, Ordering::Relaxed);
    }

    fn usage(&self) -> usize {
        self.0.load(Ordering::Relaxed).min(2) as usize
    }

}

impl AtomicTTSlicePatternEntry {

    fn is_hit(&self, packed_slice: u64) -> bool {
        self.0.load(Ordering::Relaxed) == packed_slice
    }

    // interior-mutability
    fn store_mut(&self, packed_slice: u64, pattern: &SlicePattern) {
        self.0.store(packed_slice, Ordering::Relaxed);
        self.1.store(u64::from_ne_bytes(pattern.patterns[0 ..8].try_into().unwrap()), Ordering::Relaxed);
        self.2.store(u64::from_ne_bytes(pattern.patterns[8 .. 16].try_into().unwrap()), Ordering::Relaxed);
    }

    fn decode(&self) -> SlicePattern {
        SlicePattern {
            patterns: unsafe {
                std::mem::transmute::<[u64; 2], [u8; 16]>([self.1.load(Ordering::Relaxed), self.2.load(Ordering::Relaxed)])
            }
        }
    }

}

pub struct TTSlicePatternMemo {
    table: Vec<AtomicTTSlicePatternEntry>,
}

impl Default for TTSlicePatternMemo {

    fn default() -> Self {
        let mut new = Self {
            table: Vec::new(),
        };

        new.resize_mut(16);

        new
    }

}

impl AbstractTranspositionTable<AtomicTTSlicePatternEntry> for TTSlicePatternMemo {

    fn internal_table(&self) -> &Vec<AtomicTTSlicePatternEntry> {
        &self.table
    }

    fn internal_table_mut(&mut self) -> &mut Vec<AtomicTTSlicePatternEntry> {
        &mut self.table
    }

    fn assign_internal_table_mut(&mut self, table: Vec<AtomicTTSlicePatternEntry>) {
        self.table = table;
    }

}

impl SlicePatternMemo for TTSlicePatternMemo {

    fn prefetch_memo(&self, packed_slice: u64) {
        self.prefetch(Self::build_hash_key(packed_slice));
    }

    fn probe_or_put_mut<F>(&mut self, packed_slice: u64, produce: F) -> SlicePattern
    where F: FnOnce() -> SlicePattern
    {
        let atomic_entry = {
            let slice_hash = Self::build_hash_key(packed_slice);
            let idx = self.calculate_index(slice_hash);
            &self.table[idx]
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

impl TTSlicePatternMemo {

    fn build_hash_key(packed_slice: u64) -> HashKey {
        // fibonacci-hashing
        HashKey(packed_slice.wrapping_mul(0x9e3779b97f4a7c15))
    }

}
