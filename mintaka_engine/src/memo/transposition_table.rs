use crate::memo::tt_entry::{TTEntry, TTEntryBucket};
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::memo::hash_key::HashKey;
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

impl TranspositionTable {

    pub fn new_with_size(size_in_mib: usize) -> Self {
        let mut new = Self {
            table: Vec::new(),
            age: AtomicU8::new(0),
        };

        new.resize_mut(size_in_mib);

        new
    }

    pub fn probe(&self, key: HashKey) -> Option<TTEntry> {
        let idx = self.calculate_index(key);
        let compact_key = key.0 as u32;

        self.table[idx].probe(compact_key)
    }

    pub fn store_mut(
        &mut self,
        key: HashKey,
        entry: TTEntry,
    ) {
        let idx = self.calculate_index(key);
        self.table[idx].store_mut(key.0 as u32, entry);
    }

}
