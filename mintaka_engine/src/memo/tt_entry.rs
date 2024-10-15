use mintaka::notation::pos::Pos;
use mintaka::utils::abstract_transposition_table::Clearable;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Eq, PartialEq, Default)]
#[repr(u8)]
pub enum TTFlag {
    #[default] PV,
    LOWER,
    UPPER,
    EXACT,
}

// 64 bit
pub struct TTEntry {
    pub best_move: Pos,
    pub depth: u8,
    pub flag: TTFlag,
    pub score: i16,
    pub eval: i16,
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

pub struct TTEntryBucket {
    key_pair: AtomicU64,
    hi_body: AtomicU64,
    lo_body: AtomicU64,
}

pub enum TTEntryBucketPosition {
    HI,
    LO
}

impl Clearable for TTEntryBucket {

    fn clear_mut(&mut self) {
        self.key_pair.store(0, Ordering::Relaxed);
        self.hi_body.store(0, Ordering::Relaxed);
        self.lo_body.store(0, Ordering::Relaxed);
    }

}

impl TTEntryBucket {

    pub fn probe(&self, compact_key: u32) -> Option<TTEntry> {
        let key_pair = self.key_pair.load(Ordering::Relaxed);

        if key_pair & compact_key as u64 == 0x00000000_FFFFFFFF {
            Some(TTEntry::from(self.hi_body.load(Ordering::Relaxed)))
        } else if key_pair & (compact_key as u64) << 32 == 0xFFFFFFFF_00000000 {
            Some(TTEntry::from(self.lo_body.load(Ordering::Relaxed)))
        } else { None }
    }

    pub fn store(&mut self, compact_key: u32, entry: TTEntry, pos: TTEntryBucketPosition) {
        match pos {
            TTEntryBucketPosition::HI => {
                self.key_pair.store((compact_key as u64) << 32, Ordering::Relaxed);
                self.hi_body.store(u64::from(entry), Ordering::Relaxed);
            }
            TTEntryBucketPosition::LO => {
                self.key_pair.store(compact_key as u64, Ordering::Relaxed);
                self.lo_body.store(u64::from(entry), Ordering::Relaxed);
            }
        }
    }

}
