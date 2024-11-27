use crate::memo::abstract_transposition_table::{AbstractTranspositionTable, Clearable};
use crate::memo::hash_key::HashKey;
use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::slice_pattern::SlicePattern;
use std::sync::atomic::{AtomicU64, Ordering};

// layout: block 0 = slice key 32 bits
//         block 1, 2 = black_patterns 128 bits
//         block 3, 4 = white_patterns 128 bits
struct AtomicTTSlicePatternEntry(AtomicU64, AtomicU64, AtomicU64, AtomicU64, AtomicU64);

impl Clearable for AtomicTTSlicePatternEntry {

    fn clear_mut(&mut self) {
        self.0.store(0, Ordering::Relaxed);
        self.1.store(0, Ordering::Relaxed);
        self.2.store(0, Ordering::Relaxed);
        self.3.store(0, Ordering::Relaxed);
        self.4.store(0, Ordering::Relaxed);
    }

}

impl AtomicTTSlicePatternEntry {

    fn is_hit(&self, packed_slice: u64) -> bool {
        self.0.load(Ordering::Relaxed) == packed_slice
    }

    // interior-mutability
    fn store_mut(&self, packed_slice: u64, pattern: &SlicePattern) {
        self.0.store(packed_slice, Ordering::Relaxed);
        self.1.store(u64::from_ne_bytes(pattern.black_patterns[0 ..8].try_into().unwrap()), Ordering::Relaxed);
        self.2.store(u64::from_ne_bytes(pattern.black_patterns[8 .. 16].try_into().unwrap()), Ordering::Relaxed);
        self.3.store(u64::from_ne_bytes(pattern.white_patterns[0 ..8].try_into().unwrap()), Ordering::Relaxed);
        self.4.store(u64::from_ne_bytes(pattern.white_patterns[8 .. 16].try_into().unwrap()), Ordering::Relaxed);
    }

    fn decode(&self) -> SlicePattern {
        let mut black_patterns: [u8; 16] = [0u8; 16];
        black_patterns[0 .. 8].copy_from_slice(&self.1.load(Ordering::Relaxed).to_ne_bytes());
        black_patterns[8 .. 16].copy_from_slice(&self.2.load(Ordering::Relaxed).to_ne_bytes());

        let mut white_patterns: [u8; 16] = [0u8; 16];
        white_patterns[0 .. 8].copy_from_slice(&self.3.load(Ordering::Relaxed).to_ne_bytes());
        white_patterns[8 .. 16].copy_from_slice(&self.4.load(Ordering::Relaxed).to_ne_bytes());

        SlicePattern {
            black_patterns,
            white_patterns,
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

        new.resize_mut(64);

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
        HashKey(unsafe { packed_slice.wrapping_mul(0x9e3779b97f4a7c15) })
    }

}
