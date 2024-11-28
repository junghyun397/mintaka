use crate::memo::tt_entry::{TTEntry, TTEntryBucket, TTFlag};
use mintaka::memo::abstract_transposition_table::AbstractTranspositionTable;
use mintaka::memo::hash_key::HashKey;
use mintaka::notation::node::{Eval, Score};
use mintaka::notation::pos::Pos;
use std::sync::atomic::AtomicU8;

pub struct TranspositionTable {
    table: Vec<TTEntryBucket>,
    age: AtomicU8
}

impl AbstractTranspositionTable<TTEntryBucket> for TranspositionTable {

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

impl Default for TranspositionTable {

    fn default() -> Self {
        let mut new = Self {
            table: Vec::new(),
            age: AtomicU8::new(0),
        };

        new.resize_mut(256);

        new
    }

}

impl TranspositionTable {

    pub fn probe(&self, key: HashKey) -> Option<TTEntry> {
        let idx = self.calculate_index(key);
        let compact_key = key.0 as u32;

        self.table[idx].probe(compact_key)
    }

    pub fn store_mut(
        &mut self,
        key: HashKey,
        best_move: Pos,
        depth: u8,
        flag: TTFlag,
        score: Score,
        eval: Eval,
    ) {
        let idx = self.calculate_index(key);
        let entry = TTEntry {
            best_move,
            depth,
            flag,
            score,
            eval,
        };

        self.table[idx].store_mut(key.0 as u32, entry);
    }

    pub fn hash_usage(&self) -> usize {
        self.table.iter()
            .take(1000)
            .filter(|bucket| true)
            .count()
    }

}
