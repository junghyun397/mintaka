use crate::memo::tt_entry::{ScoreKind, TTEntry, TTEntryBucket, TTFlag};
use crate::value::Depth;
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::score::{Score, Scores};
use rusty_renju::utils::byte_size::ByteSize;
use serde::{Deserialize, Serialize, Serializer};
use std::io::{Read, Write};
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

        new.resize(size);

        new
    }

    pub fn view(&self) -> TTView<'_> {
        TTView {
            table: &self.table,
            age: self.fetch_age(),
        }
    }

    // compression level: 0-9
    pub fn export(&self, compression_level: u32) -> Vec<u8> {
        let age = self.fetch_age();
        let byte_len = self.table.len() * size_of::<TTEntryBucket>();
        let byte_cap = self.table.capacity() * size_of::<TTEntryBucket>();

        let table_ptr = self.table.as_ptr() as *mut u8;

        let mut bytes = Vec::with_capacity(byte_cap + 1);
        bytes.push(age);
        bytes.extend_from_slice(unsafe { std::slice::from_raw_parts(table_ptr, byte_len) });

        let mut encoder = lz4::EncoderBuilder::new()
            .level(compression_level)
            .build(Vec::new())
            .unwrap();

        encoder.write_all(&bytes)
            .unwrap();

        let (compressed, _) = encoder.finish();
        compressed
    }

    #[allow(clippy::uninit_vec)]
    pub fn import(source: Vec<u8>) -> Result<Self, &'static str> {
        let mut decoder = lz4::Decoder::new(std::io::Cursor::new(source))
            .map_err(|_| "failed to build decoder")?;
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|_| "failed to decompress")?;

        let age = decompressed[0];
        let payload = &decompressed[1 ..];

        if !payload.len().is_multiple_of(size_of::<TTEntryBucket>()) {
            return Err("illegal payload size");
        }

        let tt_len = payload.len() / size_of::<TTEntryBucket>();

        let mut table = Vec::with_capacity(tt_len);

        unsafe {
            table.set_len(tt_len);
            std::ptr::copy_nonoverlapping(
                payload.as_ptr(),
                table.as_mut_ptr() as *mut u8,
                payload.len()
            );
        }

        Ok(Self {
            table,
            age: AtomicU8::new(age),
        })
    }

}

pub enum TTHit {
    Entry(TTEntry),
    Eval(Score),
    None
}

#[derive(Debug, Copy, Clone)]
pub struct TTView<'a> {
    table: &'a [TTEntryBucket],
    pub age: u8,
}

impl TTView<'_> {

    fn calculate_index(&self, key: HashKey) -> usize {
        ((u64::from(key) as u128 * (self.table.len() as u128)) >> 64) as usize
    }

    pub fn probe_entry(&self, key: HashKey) -> Option<TTEntry> {
        let idx = self.calculate_index(key);
        self.table[idx].probe(key)
    }

    pub fn probe(&self, key: HashKey) -> TTHit {
        match self.probe_entry(key) {
            Some(entry) => match (entry.tt_flag.maybe_score_kind(), entry.score as Score) {
                (Some(_), _) => TTHit::Entry(entry),
                (None, Score::NAN) => TTHit::None, // endgame entry
                (None, _) => TTHit::Eval(entry.eval as Score),
            },
            None => TTHit::None
        }
    }

    pub fn store_entry(&self, key: HashKey, entry: TTEntry) {
        let idx = self.calculate_index(key);
        self.table[idx].store(key, entry);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store(
        &self,
        key: HashKey,
        best_move: MaybePos,
        maybe_score_kind: Option<ScoreKind>,
        vcf_depth: Depth,
        depth: Depth,
        eval: Score,
        score: Score,
        is_pv: bool,
    ) {
        let idx = self.calculate_index(key);
        let depth = depth as u8;
        let eval = eval as i16;
        let score = score as i16;

        let bucket = &self.table[idx];

        if let Some(mut entry) = bucket.probe(key) {
            let entry_score_kind = entry.tt_flag.maybe_score_kind();

            let score_kind_value = maybe_score_kind.map_or(0, ScoreKind::into);
            let replace_score = depth + score_kind_value + 5;
            let keep_score = entry.depth + entry_score_kind.map_or(0, ScoreKind::into);

            if self.age > entry.age
                || (maybe_score_kind == Some(ScoreKind::Exact) && entry_score_kind != Some(ScoreKind::Exact))
                || replace_score > keep_score
            {
                if best_move.is_some() {
                    entry.best_move = best_move;
                }

                entry.tt_flag = TTFlag::new(maybe_score_kind, is_pv, vcf_depth);
                entry.age = self.age;
                entry.depth = depth;
                entry.eval = eval;
                entry.score = score;

                bucket.store(key, entry);
            }
        } else {
            let entry = TTEntry {
                best_move,
                tt_flag: TTFlag::new(maybe_score_kind, is_pv, vcf_depth),
                age: self.age,
                depth,
                eval,
                score,
            };

            bucket.store(key, entry);
        }
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
            use std::arch::aarch64::{_prefetch, _PREFETCH_LOCALITY3, _PREFETCH_READ};
            let idx = self.calculate_index(key);
            let entry = &self.table[idx];
            _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY3>((entry as *const TTEntryBucket).cast());
        }
    }

}

impl Serialize for TranspositionTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_bytes(&self.export(9))
    }
}

impl<'de> Deserialize<'de> for TranspositionTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let bytes = Vec::<u8>::deserialize(deserializer)?;

        Self::import(bytes)
            .map_err(serde::de::Error::custom)
    }
}
