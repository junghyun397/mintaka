use crate::value::{Depth, Eval, Score};
use rusty_renju::memo::abstract_transposition_table::AbstractTTEntry;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::Pos;
use std::sync::atomic::{AtomicU64, Ordering};

const KEY_OFFSET: usize = 21;
const KEY_MASK: u64 = 0b01_1111_1111_1111_1111_1111;

#[derive(Copy, Clone)]
pub struct TTEntryKey {
    lower_21_bits: u64
}

impl From<HashKey> for TTEntryKey {

    fn from(hash_key: HashKey) -> Self {
        Self {
            lower_21_bits: hash_key.0 & KEY_MASK
        }
    }

}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
#[repr(u8)]
pub enum TTFlag {
    #[default] PV,
    Lower,
    Upper,
    Exact,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
#[repr(u8)]
pub enum EndgameFlag {
    #[default] Unknown,
    Cold,
    Win,
    Lose
}

// 64 bit
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, align(8))]
pub struct TTEntry {
    pub best_move: Pos, // 8
    pub flag: TTFlag, // 8
    pub endgame_flag: EndgameFlag, // 8
    pub depth: Depth, // 8
    pub eval: Eval, // 16
    pub score: Score, // 16
}

impl From<TTEntry> for u64 {

    fn from(value: TTEntry) -> Self {
        unsafe { std::mem::transmute(value) }
    }

}

impl From<u64> for TTEntry {

    fn from(value: u64) -> Self {
        unsafe { std::mem::transmute(value) }
    }

}

impl TTEntry {

    pub const EMPTY: Self = Self {
        best_move: Pos::INVALID,
        flag: TTFlag::Exact,
        endgame_flag: EndgameFlag::Unknown,
        depth: 0,
        eval: 0,
        score: 0,
    };

}

// key(21 bits) * 6 = 126 bits
// entry(64 bits) * 6 = 384 bits
// total 510 bits / 512 bits
pub struct TTEntryBucket {
    hi_keys: AtomicU64,
    lo_keys: AtomicU64,
    entries: [AtomicU64; 6]
}

impl AbstractTTEntry for TTEntryBucket {

    const BUCKET_SIZE: usize = 6;

    fn clear_mut(&self) {
        self.hi_keys.store(0, Ordering::Relaxed);
        self.lo_keys.store(0, Ordering::Relaxed);

        self.entries[0].store(0, Ordering::Relaxed);
        self.entries[1].store(0, Ordering::Relaxed);
        self.entries[2].store(0, Ordering::Relaxed);
        self.entries[3].store(0, Ordering::Relaxed);
        self.entries[4].store(0, Ordering::Relaxed);
        self.entries[5].store(0, Ordering::Relaxed);
    }

    fn usage(&self) -> usize {
        let mut count = 0;

        for idx in 0 ..6 {
            count += self.entries[idx].load(Ordering::Relaxed).min(1);
        }

        count as usize
    }

}

impl TTEntryBucket {

    fn calculate_entry_index(&self, entry_key: TTEntryKey) -> usize {
        (((entry_key.lower_21_bits << 11) * 6) >> 32) as usize
    }

    fn store_key_mut(&self, entry_idx: usize, entry_key: TTEntryKey) {
        if entry_idx < 3 {
            let hi_keys = self.hi_keys.load(Ordering::Acquire);
            let bit_offset = KEY_OFFSET * entry_idx;
            let mask = KEY_MASK << bit_offset;
            let content = (hi_keys & !mask) | (entry_key.lower_21_bits << bit_offset);
            self.hi_keys.store(content, Ordering::Release);
        } else {
            let lo_keys = self.lo_keys.load(Ordering::Acquire);
            let bit_offset = KEY_OFFSET * (entry_idx - 3);
            let mask = KEY_MASK << bit_offset;
            let content = (lo_keys & !mask) | (entry_key.lower_21_bits << bit_offset);
            self.lo_keys.store(content, Ordering::Release);
        }
    }

    pub fn probe(&self, entry_key: TTEntryKey) -> Option<TTEntry> {
        let entry_idx = self.calculate_entry_index(entry_key);
        if entry_idx < 3 {
            let hi_keys = self.hi_keys.load(Ordering::Relaxed);
            if (hi_keys >> (KEY_OFFSET * entry_idx)) & KEY_MASK == entry_key.lower_21_bits {
                return Some(self.entries[entry_idx].load(Ordering::Relaxed).into())
            }
        } else {
            let lo_keys = self.lo_keys.load(Ordering::Relaxed);
            if (lo_keys >> (KEY_OFFSET * (entry_idx - 3))) & KEY_MASK == entry_key.lower_21_bits {
                return Some(self.entries[entry_idx].load(Ordering::Relaxed).into())
            }
        }

        None
    }

    pub fn store_mut(&self, entry_key: TTEntryKey, entry: TTEntry) {
        let entry_idx = self.calculate_entry_index(entry_key);
        self.store_key_mut(entry_idx, entry_key);
        self.entries[entry_idx].store(entry.into(), Ordering::Relaxed);
    }

}
