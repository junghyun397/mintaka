use rusty_renju::assert_struct_sizes;
use rusty_renju::memo::abstract_transposition_table::AbstractTTEntry;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use std::sync::atomic::{AtomicU64, Ordering};

const KEY_SIZE: usize = 21;
const KEY_SHIFT: usize = 64 - KEY_SIZE;
const KEY_MASK: u64 = !(u64::MAX << KEY_SIZE as u64);

#[derive(Copy, Clone)]
pub struct TTEntryKey {
    key: u64
}

impl From<HashKey> for TTEntryKey {

    fn from(hash_key: HashKey) -> Self {
        Self {
            key: u64::from(hash_key) >> KEY_SHIFT,
        }
    }

}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ScoreKind {
    UpperBound = 0,
    LowerBound = 1,
    Exact = 2,
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

    pub fn new(score_kind: ScoreKind, endgame_visited: bool, is_pv: bool) -> Self {
        Self((score_kind as u8) | ((endgame_visited as u8) << 2) | ((is_pv as u8) << 3))
    }

    pub fn score_kind(&self) -> ScoreKind {
        unsafe { std::mem::transmute::<u8, ScoreKind>(self.0 & 0b11) }
    }

    pub fn endgame_visited(&self) -> bool {
        (self.0 >> 2) & 0b1 == 0b1
    }

    pub fn is_pv(&self) -> bool {
        (self.0 >> 3) & 0b1 == 0b1
    }

    pub fn set_score_kind(&mut self, score_kind: ScoreKind) {
        self.0 = (self.0 & !0b11) | score_kind as u8;
    }

    pub fn set_endgame_visited(&mut self, endgame_visited: bool) {
        self.0 = (self.0 & !(0b1 << 2)) | ((endgame_visited as u8) << 2);
    }

    pub fn set_pv(&mut self, is_pv: bool) {
        self.0 = (self.0 & !(0b1 << 3)) | ((is_pv as u8) << 3);
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

    fn calculate_slot_index(entry_key: TTEntryKey) -> usize {
        (((entry_key.key) * Self::BUCKET_SIZE) >> KEY_SIZE) as usize
    }

    fn calculate_lane_shift(slot_idx: usize) -> usize {
        KEY_SIZE * (slot_idx % 3)
    }

    pub(crate) fn probe(&self, entry_key: TTEntryKey) -> Option<TTEntry> {
        let slot_idx = Self::calculate_slot_index(entry_key);
        let keys_idx = slot_idx / 3;
        let lane_shift = Self::calculate_lane_shift(slot_idx);

        let keys = self.keys[keys_idx].load(Ordering::Relaxed);
        (((keys >> lane_shift) & KEY_MASK) == entry_key.key)
            .then(|| self.entries[slot_idx].load(Ordering::Relaxed).into())
    }

    fn store_key(&self, bucket_idx: usize, entry_key: TTEntryKey) {
        let keys_idx = bucket_idx / 3;
        let lane_shift = Self::calculate_lane_shift(bucket_idx);

        let mut keys = self.keys[keys_idx].load(Ordering::Acquire);
        keys = (keys & !(KEY_MASK << lane_shift)) | (entry_key.key << lane_shift);

        self.keys[keys_idx].store(keys, Ordering::Release);
    }

    pub(crate) fn store(&self, entry_key: TTEntryKey, entry: TTEntry) {
        let slot_idx = Self::calculate_slot_index(entry_key);

        self.store_key(slot_idx, entry_key);
        self.entries[slot_idx].store(entry.into(), Ordering::Relaxed);
    }

}
