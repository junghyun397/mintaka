use std::simd::cmp::SimdPartialEq;
use std::simd::Simd;
use rusty_renju::assert_struct_sizes;
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use std::sync::atomic::{AtomicU64, Ordering};

const KEY_SIZE: usize = 21;
const KEY_MASK: u64 = !(u64::MAX << KEY_SIZE as u64);
const KEY_SHIFT: u64 = 64 - KEY_SIZE as u64;

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

impl From<ScoreKind> for i32 {
    fn from(score_kind: ScoreKind) -> Self {
        score_kind as i32
    }
}

// age(5), is_pv(1) , score_kind(2)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct TTFlag(u8);

impl TTFlag {
    pub const MAX_TT_ENDGAME_DEPTH: u8 = 0b11110;

    pub fn new(age: u8, maybe_score_kind: Option<ScoreKind>, is_pv: bool) -> Self {
        let score_kind = maybe_score_kind.map_or(0, ScoreKind::into);

        Self(score_kind | ((is_pv as u8) << 2) | age << 3)
    }

    pub fn new_simple(age: u8, score_kind: ScoreKind, is_pv: bool) -> Self {
        Self(age << 3 | ((is_pv as u8) << 2) | score_kind as u8)
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

    pub fn set_score_kind(&mut self, score_kind: ScoreKind) {
        self.0 = (self.0 & !0b11) | score_kind as u8;
    }

    pub fn set_pv(&mut self, is_pv: bool) {
        self.0 = (self.0 & !(0b1 << 2)) | ((is_pv as u8) << 2);
    }

    pub fn age(&self) -> u8 {
        self.0 >> 3
    }

    pub fn set_age(&mut self, age: u8) {
        self.0 = (self.0 & !(0b11111 << 3)) | (age << 3);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, align(8))]
pub struct TTEntry {
    pub best_move: MaybePos, // 8
    pub tt_flag: TTFlag, // 8
    pub depth: u8, // 8
    pub endgame_depth: u8, // 8
    pub eval: i16, // 16
    pub score: i16, // 16
}

impl TTEntry {
    pub const ENDGAME_PROVEN_DEPTH: u8 = u8::MAX;
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

#[derive(Debug)]
#[repr(align(64))]
pub struct TTEntryBucket {
    signatures: [AtomicU64; 2],
    entries: [AtomicU64; 6]
}

pub struct TTEntryBucketProbe {
    pub entry: TTEntry,
    pub slot: usize,
}

assert_struct_sizes!(TTEntryBucket, size=64, align=64);

impl TTEntryBucket {
    pub const BUCKET_SIZE: usize = 6;

    const SIGNATURE_SHIFTS: [u64; 8] = [0, 21, 42, 0, 21, 42, 0, 0];

    const ENTRY_HASH_MUL: u64 = 11400714819323198549;

    fn pack_hash_key(key: HashKey) -> u64 {
        u64::from(key) & KEY_MASK
    }

    pub fn load_entries(&self) -> [u64; Self::BUCKET_SIZE] {
        std::array::from_fn(|slot_idx|
            self.entries[slot_idx].load(Ordering::Relaxed)
        )
    }

    pub fn probe(&self, key: HashKey) -> Option<TTEntryBucketProbe> {
        let signature_hi = self.signatures[0].load(Ordering::Acquire);
        let signature_lo = self.signatures[1].load(Ordering::Acquire);

        let entries = self.load_entries();

        let entries = Simd::<u64, 8>::from_array([
            entries[0], entries[1], entries[2],
            entries[3], entries[4], entries[5],
            0, 0,
        ]);

        let signatures = Simd::<u64, 8>::from_array([
            signature_hi, signature_hi, signature_hi,
            signature_lo, signature_lo, signature_lo,
            0, 0,
        ]);

        let stored_signatures = (signatures >> Simd::<u64, 8>::from_array(Self::SIGNATURE_SHIFTS))
            & Simd::<u64, 8>::splat(KEY_MASK);

        let entry_checksums = (entries * Simd::<u64, 8>::splat(Self::ENTRY_HASH_MUL))
            >> KEY_SHIFT;

        let signature_matches = (
            (stored_signatures ^ entry_checksums).simd_eq(Simd::<u64, 8>::splat(Self::pack_hash_key(key)))
                & entries.simd_ne(Simd::<u64, 8>::splat(0))
        ).to_bitmask() & 0b111111;

        if signature_matches == 0 {
            None
        } else {
            let slot = signature_matches.trailing_zeros() as usize;

            Some(TTEntryBucketProbe { slot, entry: entries[slot].into() })
        }
    }

    pub fn store(&self, slot: usize, key: HashKey, entry: TTEntry) {
        let entry = u64::from(entry);

        let signature = Self::pack_hash_key(key) ^ (entry.wrapping_mul(Self::ENTRY_HASH_MUL) >> KEY_SHIFT);

        self.entries[slot].store(entry, Ordering::Relaxed);

        let shift = KEY_SIZE * (slot % 3);

        self.signatures[slot / 3].try_update(Ordering::Release, Ordering::Relaxed, |old|
            Some((old & !(KEY_MASK << shift)) | (signature << shift))
        ).unwrap();
    }

    pub fn clear(&self) {
        for signature in &self.signatures {
            signature.store(0, Ordering::Relaxed);
        }

        for entry in &self.entries {
            entry.store(0, Ordering::Relaxed);
        }
    }

    pub fn usage(&self, age: u8) -> usize {
        self.entries
            .iter()
            .map(|entry| {
                let entry = entry.load(Ordering::Relaxed);

                (entry != 0 && TTEntry::from(entry).tt_flag.age() == age) as usize
            })
            .sum()
    }
}
