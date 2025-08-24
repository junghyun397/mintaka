use crate::memo::tt_entry::{EndgameFlag, ScoreKind, TTEntry, TTEntryBucket, TTFlag};
use rusty_renju::memo::abstract_transposition_table::AbstractTranspositionTable;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::value::Score;
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

        new.resize_mut(size);

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
    pub fn store_entry(&self, key: HashKey, entry: TTEntry) {
        let idx = self.calculate_index(key);
        self.table[idx].store(key.into(), entry);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store(
        &self,
        key: HashKey,
        maybe_best_move: MaybePos,
        score_kind: ScoreKind,
        endgame_flag: EndgameFlag,
        depth: i32,
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
                entry.depth = depth as u8;
                entry.eval = eval as i16;
                entry.score = score as i16;

                self.table[idx].store(key.into(), entry);
            }
        } else {
            let entry = TTEntry {
                best_move: maybe_best_move,
                tt_flag: TTFlag::new(score_kind, endgame_flag, false),
                age: self.age,
                depth: depth as u8,
                eval: eval as i16,
                score: score as i16,
            };

            self.table[idx].store(key.into(), entry);
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
            use std::arch::aarch64::{_prefetch, _PREFETCH_LOCALITY0, _PREFETCH_READ};
            let idx = self.calculate_index(key);
            let entry = &self.table[idx];
            _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY0>((entry as *const TTEntryBucket).cast());
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
