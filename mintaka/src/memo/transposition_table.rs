use crate::memo::tt_entry::{EndgameFlag, ScoreKind, TTEntry, TTEntryBucket, TTFlag};
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::{Depth, Eval, Score};
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

    fn assign_internal_table_mut(&mut self, table: Vec<TTEntryBucket>) {
        self.table = table;
    }

}

impl TranspositionTable {

    pub fn new_with_size(size_in_kib: usize) -> Self {
        let mut new = Self {
            table: Vec::new(),
            age: AtomicU8::new(0),
        };

        new.resize_mut(size_in_kib);

        new
    }

    pub fn size_in_kib(&self) -> usize {
        size_of_val(&self.table) / 1024
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

    pub fn store_mut(
        &self,
        key: HashKey,
        ply: usize,
        best_move: Pos,
        score_kind: ScoreKind,
        endgame_flag: EndgameFlag,
        depth: Depth,
        eval: Eval,
        mut score: Score,
        is_pv: bool,
    ) {
        let idx = self.calculate_index(key);

        if let Some(mut entry) = self.table[idx].probe(key.into()) {
            if self.fetch_age() > entry.age
                || score_kind == ScoreKind::Exact
                || entry.depth.saturating_add(5) > entry.depth
            {
                if best_move != Pos::INVALID {
                    entry.best_move = best_move;
                }

                entry.tt_flag = TTFlag::new(score_kind, endgame_flag, is_pv);
                entry.age = self.fetch_age();
                entry.depth = depth;
                entry.eval = eval;
                entry.score = score;

                self.table[idx].store_mut(key.into(), entry);
            }

            return;
        }

        let entry = TTEntry {
            best_move,
            tt_flag: TTFlag::new(score_kind, endgame_flag, false),
            age: self.fetch_age(),
            depth,
            eval,
            score,
        };

        self.table[idx].store_mut(key.into(), entry);
    }

    pub fn increase_age(&self) {
        self.age.fetch_add(1, Ordering::Relaxed);
    }

    pub fn fetch_age(&self) -> u8 {
        self.age.load(Ordering::Relaxed)
    }

}
