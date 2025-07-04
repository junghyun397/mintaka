use crate::memo::tt_entry::{EndgameFlag, ScoreKind, TTEntry, TTEntryBucket, TTFlag};
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::value::Score;
use rusty_renju::utils::byte_size::ByteSize;
use std::sync::atomic::{AtomicU8, Ordering};

pub struct TranspositionTable {
    table: Vec<TTEntryBucket>,
    age: AtomicU8
}

impl AbstractTranspositionTable for TranspositionTable {

    type EntryType = TTEntryBucket;

    fn internal_table(&self) -> &Vec<TTEntryBucket> {
        &self.table
    }

    fn internal_table_mut(&mut self) -> &mut Vec<TTEntryBucket> {
        &mut self.table
    }

    fn fetch_age(&self) -> u8 {
        self.age.load(Ordering::Relaxed)
    }

    fn increase_age(&self) {
        self.age.fetch_add(1, Ordering::Relaxed);
    }

    fn clear_age(&self) {
        self.age.store(0, Ordering::Relaxed);
    }

}

impl TranspositionTable {

    pub fn new_with_size(size: ByteSize) -> Self {
        let mut new = Self {
            table: Vec::new(),
            age: AtomicU8::new(0),
        };

        new.resize_mut(size);

        new
    }

    pub fn view(&self) -> TTView<'_> {
        TTView {
            table: &self.table,
            age: self.fetch_age(),
        }
    }

}

#[derive(Debug, Copy, Clone)]
pub struct TTView<'a> {
    table: &'a [TTEntryBucket],
    pub age: u8,
}

impl TTView<'_> {

    fn calculate_index(&self, key: HashKey) -> usize {
        ((key.0 as u128 * (self.table.len() as u128)) >> 64) as usize
    }

    #[inline]
    pub fn probe(&self, key: HashKey) -> Option<TTEntry> {
        let idx = self.calculate_index(key);
        self.table[idx].probe(key.into())
    }

    #[inline]
    pub fn store_entry_mut(&self, key: HashKey, entry: TTEntry) {
        let idx = self.calculate_index(key);
        self.table[idx].store_mut(key.into(), entry);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_mut(
        &self,
        key: HashKey,
        maybe_best_move: MaybePos,
        score_kind: ScoreKind,
        endgame_flag: EndgameFlag,
        depth: u8,
        eval: Score,
        score: Score,
        is_pv: bool,
    ) {
        let idx = self.calculate_index(key);

        if let Some(mut entry) = self.table[idx].probe(key.into()) {
            if self.age > entry.age
                || score_kind == ScoreKind::Exact
                || entry.depth.saturating_add(5) > entry.depth
            {
                if maybe_best_move.is_some() {
                    entry.best_move = maybe_best_move;
                }

                entry.tt_flag = TTFlag::new(score_kind, endgame_flag, is_pv);
                entry.age = self.age;
                entry.depth = depth;
                entry.eval = eval;
                entry.score = score;

                self.table[idx].store_mut(key.into(), entry);
            }

            return;
        }

        let entry = TTEntry {
            best_move: maybe_best_move,
            tt_flag: TTFlag::new(score_kind, endgame_flag, false),
            age: self.age,
            depth,
            eval,
            score,
        };

        self.table[idx].store_mut(key.into(), entry);
    }

    pub fn prefetch(&self, key: HashKey) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
            let idx = self.calculate_index(key);
            let entry = &self.table[idx];
            _mm_prefetch::<_MM_HINT_T0>((entry as *const TTEntryBucket).cast());
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            use std::arch::aarch64::{_prefetch, _PREFETCH_LOCALITY0, _PREFETCH_READ};
            let idx = self.calculate_index(key);
            let entry = &self.table[idx];
            _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY0>((entry as *const TTEntryBucket).cast());
        }
    }

}
