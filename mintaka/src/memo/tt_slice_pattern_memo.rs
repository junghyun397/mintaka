use crate::memo::hash_key::HashKey;
use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::notation::color::Color;
use crate::slice_pattern::SlicePattern;
use crate::utils::abstract_transposition_table::{AbstractTranspositionTable, Clearable};
use std::sync::atomic::{AtomicU64, Ordering};

// layout: block 0 = (padding 16 bits + Option<(u8, Color)> 16 bits, raw slice key 32 bits)
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

    fn encode(packed_slice: u64, pattern: &SlicePattern) -> Self {
        unsafe {
            let pre_block_0 = (std::mem::transmute::<Option<(u8, Color)>, u16>(pattern.five_in_a_row) as u64) << 32;
            let block_0 = pre_block_0 | packed_slice;
            let block_1 = u64::from_ne_bytes(pattern.black_patterns[0 ..8].try_into().unwrap());
            let block_2 = u64::from_ne_bytes(pattern.black_patterns[8 .. 16].try_into().unwrap());
            let block_3 = u64::from_ne_bytes(pattern.white_patterns[0 ..8].try_into().unwrap());
            let block_4 = u64::from_ne_bytes(pattern.white_patterns[8 .. 16].try_into().unwrap());

            Self(
                AtomicU64::from(block_0),
                AtomicU64::from(block_1),
                AtomicU64::from(block_2),
                AtomicU64::from(block_3),
                AtomicU64::from(block_4),
            )
        }
    }

    fn decode(&self) -> SlicePattern {
        let five_in_a_row = unsafe {
            std::mem::transmute::<u16, Option<(u8, Color)>>(
                ((self.0.load(Ordering::Relaxed) & 0xFFFFFFFF_00000000) >> 48) as u16
            )
        };

        let mut black_patterns = [0u8; 16];
        black_patterns[0 .. 8].copy_from_slice(&self.1.load(Ordering::Relaxed).to_ne_bytes());
        black_patterns[8 .. 16].copy_from_slice(&self.2.load(Ordering::Relaxed).to_ne_bytes());

        let mut white_patterns = [0u8; 16];
        white_patterns[0 .. 8].copy_from_slice(&self.3.load(Ordering::Relaxed).to_ne_bytes());
        white_patterns[8 .. 16].copy_from_slice(&self.4.load(Ordering::Relaxed).to_ne_bytes());

        SlicePattern {
            black_patterns,
            white_patterns,
            five_in_a_row,
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

        new.resize_mut(256);

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

    fn probe_or_put_mut<F>(&mut self, packed_slice: u64, produce: F) -> SlicePattern
    where F: FnOnce() -> SlicePattern
    {
        // fibonacci-hashing
        let slice_hash = HashKey((packed_slice | (packed_slice << 32)) * 0x9e3779b97f4a7c15);
        let idx = self.calculate_index(slice_hash);
        let atomic_entry = &self.table[idx];

        let entry_block_0 = atomic_entry.0.load(Ordering::Relaxed);
        if entry_block_0 & 0x0000_0000_FFFF_FFFF == packed_slice {
            return atomic_entry.decode();
        }

        let slice_pattern = produce();
        self.table[idx] = AtomicTTSlicePatternEntry::encode(
            packed_slice,
            &slice_pattern
        );

        slice_pattern
    }

}
