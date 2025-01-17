use crate::value::{Depth, Eval, Score};
use rusty_renju::memo::abstract_transposition_table::AbstractTTEntry;
use rusty_renju::notation::pos::Pos;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Eq, PartialEq, Default)]
#[repr(u8)]
pub enum TTFlag {
    #[default] PV,
    Lower,
    Upper,
    Exact,
}

#[derive(Eq, PartialEq, Default)]
#[repr(u8)]
pub enum EndgameFlag {
    #[default] Unknown,
    Cold,
    Win,
    Lose
}

// 64 bit
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

const HI_KEY_MASK: u64 = 0x0000_0000_FFFF_FFFF;
const LO_KEY_MASK: u64 = 0xFFFF_FFFF_0000_0000;

pub struct TTEntryBucket {
    key_pair: AtomicU64,
    hi_body: AtomicU64,
    lo_body: AtomicU64,
}

impl AbstractTTEntry for TTEntryBucket {

    fn clear_mut(&mut self) {
        self.key_pair.store(0, Ordering::Relaxed);
        self.hi_body.store(0, Ordering::Relaxed);
        self.lo_body.store(0, Ordering::Relaxed);
    }

    fn usage(&self) -> usize {
        let key_pair = self.key_pair.load(Ordering::Relaxed);
        let mut count = 0;

        if key_pair & HI_KEY_MASK != 0 {
            count += 1;
        }

        if key_pair & LO_KEY_MASK != 0 {
            count += 1;
        }

        count
    }

}

impl TTEntryBucket {

    pub fn probe(&self, compact_key: u32) -> Option<TTEntry> {
        let key_pair = self.key_pair.load(Ordering::Relaxed);

        if key_pair & HI_KEY_MASK == compact_key as u64 {
            Some(TTEntry::from(self.hi_body.load(Ordering::Relaxed)))
        } else if key_pair >> 32 == compact_key as u64 {
            Some(TTEntry::from(self.lo_body.load(Ordering::Relaxed)))
        } else { None }
    }

    pub fn store_mut(&self, compact_key: u32, entry: TTEntry) {
        let key_pair = self.key_pair.load(Ordering::Relaxed);

        if key_pair == 0 || key_pair & HI_KEY_MASK == compact_key as u64 {
            self.key_pair.store((key_pair & HI_KEY_MASK) | (compact_key as u64), Ordering::Relaxed);
            self.hi_body.store(u64::from(entry), Ordering::Relaxed);
        } else {
            self.key_pair.store((key_pair & LO_KEY_MASK) | ((compact_key as u64) << 32), Ordering::Relaxed);
            self.lo_body.store(u64::from(entry), Ordering::Relaxed);
        }
    }

}
