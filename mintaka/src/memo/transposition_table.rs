use crate::memo::tt_entry::{ScoreKind, TTEntry, TTEntryBucket, TTEntryBucketProbe, TTFlag};
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::score::{MaybeScore, Score};
use rusty_renju::utils::byte_size::ByteSize;
use std::fmt::{Debug, Display};
#[cfg(feature = "compress-tt")]
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;
use crate::utils::depth::Depth;

pub struct TranspositionTable {
    table: Vec<TTEntryBucket>,
    age: AtomicU8,
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

    pub fn size(&self) -> ByteSize {
        ByteSize::from_bytes((self.table.len() * size_of::<TTEntryBucket>()) as u64)
    }

    fn calculate_table_len(size: ByteSize) -> usize {
        size.bytes() as usize / size_of::<TTEntryBucket>()
    }

    pub fn fetch_age(&self) -> u8 {
        self.age.load(Ordering::Relaxed)
    }

    pub fn increase_age(&self) {
        self.age.try_update(Ordering::Acquire, Ordering::Relaxed, |age|
            Some(age.wrapping_add(1) & TTFlag::MAX_AGE)
        ).unwrap();
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
        const ENTRY_SIZE: f64 = size_of::<TTEntryBucket>() as f64 * TTEntryBucket::BUCKET_SIZE as f64;

        let total_nodes = nps as f64 * expected_runtime.as_millis() as f64 / 1000.0;
        ByteSize::from_bytes((total_nodes / ENTRY_SIZE * FILL_FACTOR) as u64)
    }

    pub fn hash_full_permille(&self, reference_age: u8) -> usize {
        const SAMPLE: usize = 1000;

        let used: usize = self.table.iter()
            .take(SAMPLE)
            .map(|entry| entry.usage(reference_age))
            .sum();

        used * 1000 / (SAMPLE * TTEntryBucket::BUCKET_SIZE)
    }

    // compression level: 0-9
    pub fn export(&self, compression_level: u32) -> Vec<u8> {
        let age = (self.fetch_age() as u64).to_be_bytes();
        let byte_len = self.table.len() * size_of::<TTEntryBucket>();
        let byte_cap = self.table.capacity() * size_of::<TTEntryBucket>();

        let table_ptr = self.table.as_ptr() as *mut u8;

        let mut bytes = Vec::with_capacity(byte_cap + 8);
        bytes.extend(age);
        bytes.extend_from_slice(unsafe { std::slice::from_raw_parts(table_ptr, byte_len) });
        tt_compress(&bytes, compression_level)
    }

    #[allow(clippy::uninit_vec)]
    pub fn import(source: Vec<u8>) -> Result<Self, TTImportError> {
        let decompressed = tt_decompress(source)?;

        let age = (&decompressed[0..8])
            .try_into()
            .map(u64::from_be_bytes)
            .unwrap_or_default()
            as u8;

        let payload = &decompressed[8..];

        if !payload.len().is_multiple_of(size_of::<TTEntryBucket>()) {
            return Err(TTImportError::BrokenPayload);
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
    fn relative_age(&self, age: u8) -> u8 {
        self.age.wrapping_sub(age) & TTFlag::MAX_AGE
    }

    fn calculate_index(&self, key: HashKey) -> usize {
        ((u64::from(key) as u128 * (self.table.len() as u128)) >> 64) as usize
    }

    pub fn probe(&self, key: HashKey) -> Option<TTEntryBucketProbe> {
        let idx = self.calculate_index(key);
        self.table[idx].probe(key)
    }

    pub fn update_entry(&self, key: HashKey, slot: usize, entry: TTEntry) {
        let idx = self.calculate_index(key);

        self.table[idx].store(slot, key, entry);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store(
        &self,
        key: HashKey,
        best_move: MaybePos,
        depth: Depth,
        endgame_depth: u8,
        maybe_score_kind: Option<ScoreKind>,
        eval: MaybeScore,
        score: MaybeScore,
        is_pv: bool,
    ) {
        let idx = self.calculate_index(key);

        let bucket = &self.table[idx];

        if let Some(TTEntryBucketProbe { slot, entry: exist_entry }) = bucket.probe(key) {
            if self.age != exist_entry.tt_flag.age()
                || maybe_score_kind == Some(ScoreKind::Exact)
                || depth.value() as u8 + 3 + 4 * is_pv as u8 > exist_entry.depth
            {
                let entry = TTEntry {
                    best_move: best_move.or(exist_entry.best_move),
                    tt_flag: TTFlag::new(self.age, maybe_score_kind, is_pv),
                    depth: depth.value() as u8,
                    endgame_depth,
                    eval: eval.or(MaybeScore::from(exist_entry.eval as i32)).unwrap_unchecked() as i16,
                    score: score.or(MaybeScore::from(exist_entry.score as i32)).unwrap_unchecked() as i16,
                };

                bucket.store(slot, key, entry);
            }
        } else {
            let entries = bucket.load_entries();

            let victim_slot =
                if let Some(slot) = entries.iter().position(|&entry| entry == 0) {
                    slot
                } else {
                    entries
                        .map(|entry| {
                            let entry = TTEntry::from(entry);

                            entry.depth as i32
                                + entry.endgame_depth as i32 / 20
                                - 6 * self.relative_age(entry.tt_flag.age()) as i32
                        })
                        .into_iter()
                        .enumerate()
                        .min_by_key(|(_, score)| *score)
                        .unwrap().0
                };

            let entry = TTEntry {
                best_move,
                tt_flag: TTFlag::new(self.age, maybe_score_kind, is_pv),
                depth: depth.value() as u8,
                endgame_depth,
                eval: eval.unwrap_unchecked() as i16,
                score: score.unwrap_unchecked() as i16,
            };

            bucket.store(victim_slot, key, entry);
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
            _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY3>(
                (entry as *const TTEntryBucket).cast(),
            );
        }
    }
}

pub fn encode_mate_distance(score: Score, ply: usize) -> Score {
    if score.is_mate() {
        let score = score.value();

        Score::from_i32(score + ply as i32 * score.signum())
    } else {
        score
    }
}

pub fn decode_mate_distance(score: Score, ply: usize) -> Score {
    if score.is_mate() {
        let score = score.value();

        Score::from_i32(score - ply as i32 * score.signum())
    } else {
        score
    }
}

#[derive(Debug)]
pub enum TTImportError {
    FailedDecompress,
    BrokenPayload,
}

impl Display for TTImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TTImportError::FailedDecompress => write!(f, "failed to decompress"),
            TTImportError::BrokenPayload => write!(f, "broken payload"),
        }
    }
}

impl std::error::Error for TTImportError {}

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
fn tt_decompress(source: Vec<u8>) -> Result<Vec<u8>, TTImportError> {
    let mut decoder =
        lz4::Decoder::new(std::io::Cursor::new(source)).map_err(|_| TTImportError::FailedDecompress)?;

    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|_| TTImportError::FailedDecompress)?;

    Ok(decompressed)
}

#[cfg(not(feature = "compress-tt"))]
fn tt_decompress(source: Vec<u8>) -> Result<Vec<u8>, TTImportError> {
    Ok(source)
}
