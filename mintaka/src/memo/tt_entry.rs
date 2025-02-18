use rusty_renju::assert_struct_sizes;
use rusty_renju::memo::abstract_transposition_table::AbstractTTEntry;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::{Depth, Eval, Score};
use std::sync::atomic::{AtomicU64, Ordering};

const KEY_SIZE: usize = 21;
const KEY_MASK: u64 = !(u64::MAX << KEY_SIZE as u64);

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
pub enum ScoreKind {
    #[default] None = 0,
    Upper = 1,
    Lower = 2,
    Exact = 3,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
#[repr(u8)]
pub enum EndgameFlag {
    #[default] Unknown = 0,
    Cold = 1,
    Win = 2,
    Lose = 3,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TTFlag(u8);

impl Default for TTFlag {

    fn default() -> Self {
        Self::DEFAULT
    }

}

impl TTFlag {

    const DEFAULT: Self = Self(0);

    pub fn new(score_kind: ScoreKind, endgame_flag: EndgameFlag, is_pv: bool) -> Self {
        Self((score_kind as u8) | ((endgame_flag as u8) << 2) | ((is_pv as u8) << 4))
    }

    pub fn score_kind(&self) -> ScoreKind {
        unsafe { std::mem::transmute(self.0 & 0b11) }
    }

    pub fn endgame_flag(&self) -> EndgameFlag {
        unsafe { std::mem::transmute((self.0 >> 2) & 0b11) }
    }

    pub fn is_pv(&self) -> bool {
        (self.0 >> 4) & 1 == 1
    }

    pub fn set_score_kind(&mut self, score_kind: ScoreKind) {
        self.0 = (self.0 & !0b11) | score_kind as u8;
    }

    pub fn set_endgame_flag(&mut self, endgame_flag: EndgameFlag) {
        self.0 = (self.0 & !(0b11 << 2)) | ((endgame_flag as u8) << 2);
    }

    pub fn set_pv(&mut self, is_pv: bool) {
        self.0 = (self.0 & !(1 << 4)) | ((is_pv as u8) << 4);
    }

}

// 64 bit
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, align(8))]
pub struct TTEntry {
    pub best_move: Pos, // 8
    pub tt_flag: TTFlag, // 8
    pub age: u8, // 8
    pub depth: Depth, // 8
    pub eval: Eval, // 16
    pub score: Score, // 16
}

assert_struct_sizes!(TTEntry, size=8, align=8);

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
        tt_flag: TTFlag::DEFAULT,
        age: 0,
        depth: 0,
        eval: 0,
        score: 0,
    };

}

pub struct TTEntryBucket {
    hi_keys: AtomicU64,
    lo_keys: AtomicU64,
    entries: [AtomicU64; 6]
}

assert_struct_sizes!(TTEntryBucket, size=64, align=8);

impl AbstractTTEntry for TTEntryBucket {

    const BUCKET_SIZE: usize = 6;

    fn clear_mut(&self) {
        self.hi_keys.store(0, Ordering::Relaxed);
        self.lo_keys.store(0, Ordering::Relaxed);

        for entry in &self.entries {
            entry.store(0, Ordering::Relaxed);
        }
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

    #[inline(always)]
    fn calculate_entry_index(entry_key: TTEntryKey) -> usize {
        (((entry_key.lower_21_bits << 11) * 6) >> 32) as usize
    }

    #[inline(always)]
    fn calculate_key_content(keys: u64, lower_21_bits: u64, internal_idx: usize) -> u64 {
        let bit_offset = KEY_SIZE * internal_idx;
        let mask = KEY_MASK << bit_offset;
        (keys & !mask) | (lower_21_bits << bit_offset)
    }

    #[inline]
    fn store_key_mut(&self, entry_idx: usize, entry_key: TTEntryKey) {
        if entry_idx < 3 {
            let content = Self::calculate_key_content(
                self.hi_keys.load(Ordering::Acquire),
                entry_key.lower_21_bits, entry_idx
            );
            self.hi_keys.store(content, Ordering::Release);
        } else {
            let content = Self::calculate_key_content(
                self.lo_keys.load(Ordering::Acquire),
                entry_key.lower_21_bits, entry_idx - 3
            );
            self.lo_keys.store(content, Ordering::Release);
        }
    }

    #[inline]
    pub fn probe(&self, entry_key: TTEntryKey) -> Option<TTEntry> {
        let entry_idx = Self::calculate_entry_index(entry_key);
        let (keys, internal_idx) = if entry_idx < 3 {
            (self.hi_keys.load(Ordering::Relaxed), entry_idx)
        } else {
            (self.lo_keys.load(Ordering::Relaxed), entry_idx - 3)
        };

        ((keys >> (KEY_SIZE * internal_idx)) & KEY_MASK == entry_key.lower_21_bits)
            .then(|| self.entries[entry_idx].load(Ordering::Relaxed).into())
    }

    #[inline]
    pub fn store_mut(&self, entry_key: TTEntryKey, entry: TTEntry) {
        let entry_idx = Self::calculate_entry_index(entry_key);
        self.store_key_mut(entry_idx, entry_key);
        self.entries[entry_idx].store(entry.into(), Ordering::Relaxed);
    }

}
