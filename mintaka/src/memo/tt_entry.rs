use rusty_renju::assert_struct_sizes;
use rusty_renju::memo::abstract_transposition_table::AbstractTTEntry;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ScoreKind {
    LowerBound = 0,
    UpperBound = 1,
    Exact = 2,
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
        self.0 = (self.0 & !(0b1 << 4)) | ((is_pv as u8) << 4);
    }

}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, align(8))]
pub struct TTEntry {
    pub best_move: MaybePos, // 8
    pub tt_flag: TTFlag, // 8
    pub age: u8, // 8
    pub depth: u8, // 8
    pub eval: i16, // 16
    pub score: i16, // 16
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
        best_move: MaybePos::NONE,
        tt_flag: TTFlag::DEFAULT,
        age: 0,
        depth: 0,
        eval: 0,
        score: 0,
    };

}

#[derive(Debug)]
#[repr(align(64))]
pub struct TTEntryBucket {
    keys: [AtomicU64; 2],
    entries: [AtomicU64; 6]
}

assert_struct_sizes!(TTEntryBucket, size=64, align=64);

impl AbstractTTEntry for TTEntryBucket {

    const BUCKET_SIZE: u64 = 6;

    fn clear_mut(&self) {
        for keys in &self.keys {
            keys.store(0, Ordering::Relaxed);
        }

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
        (((entry_key.lower_21_bits << 11) * Self::BUCKET_SIZE) >> 32) as usize
    }

    #[inline]
    fn store_key(&self, bucket_idx: usize, entry_key: TTEntryKey) {
        let bit_offset = KEY_SIZE * (bucket_idx % 3);
        let key_position = bucket_idx / 3;
        let mask = KEY_MASK << bit_offset;

        let original_keys = self.keys[key_position].load(Ordering::Acquire);
        let keys = (original_keys & !mask) | (entry_key.lower_21_bits << bit_offset);

        self.keys[key_position].store(keys, Ordering::Release);
    }

    #[inline]
    pub fn probe(&self, entry_key: TTEntryKey) -> Option<TTEntry> {
        let bucket_idx = Self::calculate_entry_index(entry_key);

        let keys = self.keys[bucket_idx / 3].load(Ordering::Relaxed);
        ((keys >> (KEY_SIZE * (bucket_idx % 3))) & KEY_MASK == entry_key.lower_21_bits)
            .then(|| self.entries[bucket_idx].load(Ordering::Relaxed).into())
    }

    #[inline]
    pub fn store(&self, entry_key: TTEntryKey, entry: TTEntry) {
        let bucket_idx = Self::calculate_entry_index(entry_key);

        self.store_key(bucket_idx, entry_key);
        self.entries[bucket_idx].store(entry.into(), Ordering::Relaxed);
    }

}
