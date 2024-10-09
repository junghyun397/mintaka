use crate::memo::tt_entry::{TTEntry, TTEntryBucket, TTFlag};
use mintaka::memo::hash_key::HashKey;
use mintaka::notation::pos::Pos;
use mintaka::utils::abstract_transposition_table::AbstractTranspositionTable;
use std::sync::atomic::AtomicUsize;

pub struct TranspositionTable {
    table: Vec<TTEntryBucket>,
    age: AtomicUsize
}

impl AbstractTranspositionTable<TTEntryBucket> for TranspositionTable {

    fn internal_table(&self) -> &Vec<TTEntryBucket> {
        &self.table
    }

    fn assign_internal_table_mut(&mut self, table: Vec<TTEntryBucket>) {
        self.table = table;
    }

}

impl TranspositionTable {
    
    fn calculate_index(&self, key: HashKey) -> usize {
        self.calculate_index_u128(key.0 as u128)
    }

    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
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
        score: i16,
        eval: i16,
    ) {
        let idx = self.calculate_index(key);
        let compact_key = key.0 as u32;
        let bucket = &self.table[idx];
    }

    pub fn hash_usage(&self) -> usize {
        self.table.iter()
            .take(1000)
            .filter(|bucket| false)
            .count()
    }

}
