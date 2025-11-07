use crate::value::Depth;
use rusty_renju::assert_struct_sizes;
use rusty_renju::memo::abstract_transposition_table::AbstractTTEntry;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::score::{Score, Scores};
use std::sync::atomic::{AtomicU64, Ordering};

const KEY_SIZE: usize = 21;
const KEY_MASK: u64 = !(u64::MAX << KEY_SIZE as u64);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ScoreKind {
    UpperBound = 1,
    LowerBound = 2,
    Exact = 3,
}

impl From<ScoreKind> for u8 {
    fn from(score_kind: ScoreKind) -> Self {
        score_kind as u8
    }
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

    pub const MAX_TT_ENDGAME_DEPTH: Depth = 0b11111;

    pub fn new(maybe_score_kind: Option<ScoreKind>, is_pv: bool, endgame_depth: Depth) -> Self {
        let score_kind = maybe_score_kind.map_or(0, ScoreKind::into);
        let tt_endgame_depth = Self::clamp_endgame_depth(endgame_depth);

        Self(score_kind | ((is_pv as u8) << 2) | tt_endgame_depth << 3)
    }

    pub fn maybe_score_kind(&self) -> Option<ScoreKind> {
        let source = self.0 & 0b11;

        (source != 0).then(|| unsafe { std::mem::transmute::<u8, ScoreKind>(source) })
    }

    pub fn score_kind(&self) -> ScoreKind {
        let source = self.0 & 0b11;

        debug_assert_ne!(source, 0);

        unsafe { std::mem::transmute::<u8, ScoreKind>(self.0 & 0b11) }
    }

    pub fn is_pv(&self) -> bool {
        (self.0 >> 2) & 0b1 == 0b1
    }

    pub fn endgame_depth(&self) -> Depth {
        (self.0 >> 3) as Depth
    }

    pub fn set_score_kind(&mut self, score_kind: ScoreKind) {
        self.0 = (self.0 & !0b11) | score_kind as u8;
    }

    pub fn set_pv(&mut self, is_pv: bool) {
        self.0 = (self.0 & !(0b1 << 2)) | ((is_pv as u8) << 2);
    }

    pub fn set_endgame_depth(&mut self, endgame_depth: Depth) {
        let tt_endgame_depth = Self::clamp_endgame_depth(endgame_depth);

        self.0 = (self.0 & !(0b11111 << 3)) | (tt_endgame_depth << 3);
    }

    fn clamp_endgame_depth(endgame_depth: Depth) -> u8 {
        endgame_depth.clamp(0, Self::MAX_TT_ENDGAME_DEPTH) as u8
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

    pub fn eval(&self) -> Score {
        let eval = self.eval as Score;

        if eval == Score::NAN { 0 } else { eval }
    }

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

    fn pack_hash_key(key: HashKey) -> u64 {
        u64::from(key) & KEY_MASK
    }

    fn calculate_slot_index(packed_key: u64) -> usize {
        ((packed_key * Self::BUCKET_SIZE) >> KEY_SIZE) as usize
    }

    fn calculate_lane_shift(slot_idx: usize) -> usize {
        KEY_SIZE * (slot_idx % 3)
    }

    pub(crate) fn probe(&self, key: HashKey) -> Option<TTEntry> {
        let packed_key = Self::pack_hash_key(key);

        let slot_idx = Self::calculate_slot_index(packed_key);
        let keys_idx = slot_idx / 3;
        let lane_shift = Self::calculate_lane_shift(slot_idx);

        let keys = self.keys[keys_idx].load(Ordering::Relaxed);
        (((keys >> lane_shift) & KEY_MASK) == packed_key)
            .then(|| self.entries[slot_idx].load(Ordering::Relaxed).into())
    }

    fn store_key(&self, slot_idx: usize, masked_key: u64) {
        let keys_idx = slot_idx / 3;
        let lane_shift = Self::calculate_lane_shift(slot_idx);

        let mut keys = self.keys[keys_idx].load(Ordering::Acquire);
        keys = (keys & !(KEY_MASK << lane_shift)) | (masked_key << lane_shift);

        self.keys[keys_idx].store(keys, Ordering::Release);
    }

    pub(crate) fn store(&self, key: HashKey, entry: TTEntry) {
        let packed_key = Self::pack_hash_key(key);

        let slot_idx = Self::calculate_slot_index(packed_key);

        self.store_key(slot_idx, packed_key);
        self.entries[slot_idx].store(entry.into(), Ordering::Relaxed);
    }

}
