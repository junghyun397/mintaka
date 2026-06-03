use crate::memo::tt_entry::{ScoreKind, TTEntry, TTEntryBucket, TTFlag};
use crate::value::{Depth, Depths};
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::notation::score::{Score, Scores};
use rusty_renju::utils::byte_size::ByteSize;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, Serializer};
#[cfg(feature = "compress-tt")]
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

#[cfg(feature = "compress-tt")]
fn tt_compress(bytes: &[u8], compression_level: u32) -> Vec<u8> {
    let mut encoder = lz4::EncoderBuilder::new()
        .level(compression_level)
        .build(Vec::new())
        .unwrap();

    encoder.write_all(bytes).unwrap();

    let (compressed, _) = encoder.finish();
    compressed
}

#[cfg(not(feature = "compress-tt"))]
fn tt_compress(bytes: &[u8], _compression_level: u32) -> Vec<u8> {
    bytes.to_vec()
}

#[cfg(feature = "compress-tt")]
fn tt_decompress(source: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    let mut decoder =
        lz4::Decoder::new(std::io::Cursor::new(source)).map_err(|_| "failed to build decoder")?;
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|_| "failed to decompress")?;
    Ok(decompressed)
}

#[cfg(not(feature = "compress-tt"))]
fn tt_decompress(source: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    Ok(source)
}

pub struct TranspositionTable {
    table: Vec<TTEntryBucket>,
    age: AtomicU32,
}

impl TranspositionTable {
    pub fn new_with_size(size: ByteSize) -> Self {
        let mut new = Self {
            table: Vec::new(),
            age: AtomicU32::new(0),
        };

        new.resize(size);

        new
    }

    pub fn size(&self) -> ByteSize {
        ByteSize::from_bytes((self.table.len() * size_of::<TTEntryBucket>()) as u64)
    }

    fn calculate_table_len(size: ByteSize) -> usize {
        size.bytes() as usize / size_of::<TTEntryBucket>()
    }

    pub fn fetch_age(&self) -> u32 {
        self.age.load(Ordering::Relaxed)
    }

    pub fn increase_age(&self) {
        self.age.fetch_add(1, Ordering::Relaxed);
    }

    pub fn clear_age(&self) {
        self.age.store(0, Ordering::Relaxed);
    }

    pub fn clear(&self) {
        self.clear_age();

        for entry in self.table.iter() {
            entry.clear();
        }
    }

    pub fn resize(&mut self, size: ByteSize) {
        self.clear_age();

        let len = Self::calculate_table_len(size);

        self.table = Vec::new();

        unsafe {
            self.table = Vec::from_raw_parts(
                std::alloc::alloc_zeroed(
                    std::alloc::Layout::array::<TTEntryBucket>(len).unwrap()
                ).cast(),
                len, len
            );
        };
    }

    pub fn view(&self) -> TTView<'_> {
        TTView {
            table: &self.table,
            age: self.fetch_age(),
        }
    }

    pub fn optimal_size(nps: usize, expected_runtime: Duration) -> ByteSize {
        const FILL_FACTOR: f64 = 0.75;
        const ENTRY_SIZE: f64 = size_of::<TTEntryBucket>() as f64 / TTEntryBucket::BUCKET_SIZE as f64;

        let total_nodes = nps as f64 * expected_runtime.as_secs() as f64;
        ByteSize::from_bytes((total_nodes / ENTRY_SIZE * FILL_FACTOR) as u64)
    }

    pub fn hash_full_permille(&self) -> usize {
        const SAMPLE: usize = 1000;

        let used: usize = self.table.iter()
            .take(SAMPLE)
            .map(|entry| entry.usage(self.age.load(Ordering::Relaxed) as u8))
            .sum();

        used * 1000 / SAMPLE * TTEntryBucket::BUCKET_SIZE as usize
    }

    // compression level: 0-9
    pub fn export(&self, compression_level: u32) -> Vec<u8> {
        let age = self.fetch_age().to_be_bytes();
        let byte_len = self.table.len() * size_of::<TTEntryBucket>();
        let byte_cap = self.table.capacity() * size_of::<TTEntryBucket>();

        let table_ptr = self.table.as_ptr() as *mut u8;

        let mut bytes = Vec::with_capacity(byte_cap + 1);
        bytes.extend(age);
        bytes.extend_from_slice(unsafe { std::slice::from_raw_parts(table_ptr, byte_len) });
        tt_compress(&bytes, compression_level)
    }

    #[allow(clippy::uninit_vec)]
    pub fn import(source: Vec<u8>) -> Result<Self, &'static str> {
        let decompressed = tt_decompress(source)?;

        let age: u32 = (&decompressed[0..4])
            .try_into()
            .map(u32::from_be_bytes)
            .unwrap_or_default();

        let payload = &decompressed[4..];

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
                payload.len(),
            );
        }

        Ok(Self {
            table,
            age: AtomicU32::new(age),
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TTView<'a> {
    table: &'a [TTEntryBucket],
    pub age: u32,
}

impl TTView<'_> {
    fn calculate_index(&self, key: HashKey) -> usize {
        ((u64::from(key) as u128 * (self.table.len() as u128)) >> 64) as usize
    }

    pub fn probe(&self, key: HashKey) -> Option<TTEntry> {
        let idx = self.calculate_index(key);
        self.table[idx].probe(key)
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
        endgame_depth: u8,
        depth: Depth,
        eval: Score,
        score: Score,
        is_pv: bool,
    ) {
        let idx = self.calculate_index(key);
        let eval = eval as i16;
        let score = score as i16;

        let bucket = &self.table[idx];

        if let Some(mut entry) = bucket.probe(key) {
            if entry.tt_flag.is_endgame_proven() {
                return;
            }

            let entry_score_kind = entry.tt_flag.maybe_score_kind();

            let score_kind_value = maybe_score_kind.map_or(0, ScoreKind::into);
            let replace_score = depth + score_kind_value + 5;
            let keep_score = entry.depth as i32 + entry_score_kind.map_or(0, ScoreKind::into);

            if self.age > entry.age as u32
                || (maybe_score_kind == Some(ScoreKind::Exact)
                    && entry_score_kind != Some(ScoreKind::Exact))
                || replace_score > keep_score
            {
                if best_move.is_some() {
                    entry.best_move = best_move;
                }

                entry.tt_flag = TTFlag::new(maybe_score_kind, is_pv, endgame_depth);
                entry.age = self.age as u8;
                entry.depth = clamp_depth(depth);
                entry.eval = eval;
                entry.score = score;

                bucket.store(key, entry);
            }
        } else {
            let entry = TTEntry {
                best_move,
                tt_flag: TTFlag::new(maybe_score_kind, is_pv, endgame_depth),
                age: self.age as u8,
                depth: clamp_depth(depth),
                eval,
                score,
            };

            bucket.store(key, entry);
        }
    }

    pub fn store_endgame_proven(
        &self,
        key: HashKey,
        response_pos: Pos,
        score_kind: ScoreKind,
        score: Score,
        is_pv: bool,
    ) {
        let idx = self.calculate_index(key);

        let bucket = &self.table[idx];

        if let Some(mut entry) = bucket.probe(key) {
            if !entry.tt_flag.is_endgame_proven() {
                entry.best_move = response_pos.into();
                entry.tt_flag.set_endgame_proven();
                entry.tt_flag.set_score_kind(score_kind);
                entry.age = self.age as u8;
                entry.depth = Depth::PLY_LIMIT as u8;
                entry.score = score as i16;

                bucket.store(key, entry);
            }
        } else {
            let entry = TTEntry {
                best_move: response_pos.into(),
                tt_flag: TTFlag::new_endgame_proven(score_kind, is_pv),
                age: self.age as u8,
                depth: Depth::PLY_LIMIT as u8,
                eval: 0,
                score: score as i16,
            };

            bucket.store(key, entry);
        }
    }

    pub fn prefetch(&self, key: HashKey) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::{_MM_HINT_T0, _mm_prefetch};
            let idx = self.calculate_index(key);
            let entry = &self.table[idx];
            _mm_prefetch::<_MM_HINT_T0>((entry as *const TTEntryBucket).cast());
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            use std::arch::aarch64::{_PREFETCH_LOCALITY3, _PREFETCH_READ, _prefetch};
            let idx = self.calculate_index(key);
            let entry = &self.table[idx];
            _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY3>(
                (entry as *const TTEntryBucket).cast(),
            );
        }
    }
}

fn clamp_depth(depth: Depth) -> u8 {
    depth.clamp(0, u8::MAX as Depth) as u8
}

pub fn encode_mate_distance(score: Score, ply: usize) -> Score {
    if Score::is_mate(score) {
        score + (ply as Score) * score.signum()
    } else {
        score
    }
}

pub fn decode_mate_distance(score: Score, ply: usize) -> Score {
    if Score::is_mate(score) {
        score - (ply as Score) * score.signum()
    } else {
        score
    }
}

#[cfg(feature = "serde")]
impl Serialize for TranspositionTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.export(9))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for TranspositionTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;

        Self::import(bytes).map_err(serde::de::Error::custom)
    }
}
