use crate::memo::tt_entry::{TTEntry, TTEntryBucket, TTEntryBucketPosition, TTFlag};
use mintaka::memo::hash_key::HashKey;
use mintaka::notation::pos::Pos;
use mintaka::utils::abstract_transposition_table::AbstractTranspositionTable;
use std::sync::atomic::AtomicU8;

#[derive(Default)]
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

    pub fn probe(&self, key: HashKey) -> Option<TTEntry> {
        let idx = self.calculate_index(key);
        let compact_key = key.0 as u32;

        self.table[idx].probe(compact_key).map(|(_, entry)| entry)
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
        let lower_half_key = key.0 as u32;
        let bucket = &mut self.table[idx];

        if let Some((position, entry)) = bucket.probe(lower_half_key) {
            // replace entry
            let new_entry = TTEntry {
                best_move,
                depth,
                flag,
                score,
                eval
            };

            bucket.store(lower_half_key, position, new_entry);
        } else {
            let new_entry = TTEntry {
                best_move,
                depth,
                flag,
                score,
                eval,
            };

            bucket.store(lower_half_key, TTEntryBucketPosition::LO, new_entry);
        }
    }

    pub fn hash_usage(&self) -> usize {
        self.table.iter()
            .take(1000)
            .filter(|bucket| true)
            .count()
    }

}
