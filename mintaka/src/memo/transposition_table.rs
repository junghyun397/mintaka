use crate::memo::tt_entry::{EndgameFlag, TTEntry, TTEntryBucket, TTFlag};
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

    pub fn probe(&self, key: HashKey) -> Option<TTEntry> {
        let idx = self.calculate_index(key);
        self.table[idx].probe(key.into())
    }

    pub fn store_entry_mut(&self, key: HashKey, entry: TTEntry) {
        let idx = self.calculate_index(key);
        self.table[idx].store_mut(key.into(), entry);
    }

    pub fn store_mut(
        &self,
        key: HashKey,
        ply: usize,
        best_move: Pos,
        flag: TTFlag,
        endgame_flag: EndgameFlag,
        depth: Depth,
        eval: Eval,
        mut score: Score
    ) {
        let idx = self.calculate_index(key);
        let entry = self.table[idx].probe(key.into()).unwrap_or(TTEntry::EMPTY);

        let new_entry = TTEntry {
            best_move,
            flag,
            endgame_flag,
            depth,
            eval,
            score,
        };

        self.table[idx].store_mut(key.into(), new_entry);
    }

    pub fn view(&self) -> TTView {
        TTView {
            table_view: &self.table,
        }
    }

    pub fn increase_age(&self) {
        self.age.fetch_add(1, Ordering::Relaxed);
    }

    pub fn fetch_age(&self) -> u8 {
        self.age.load(Ordering::Relaxed)
    }

}

pub struct TTView<'a> {
    table_view: &'a [TTEntryBucket],
}

impl TTView<'_> {

}
