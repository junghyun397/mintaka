use crate::utils::byte_size::ByteSize;

pub trait AbstractTTEntry : Sync + Send {

    const BUCKET_SIZE: u64;

    fn clear_mut(&self);

    fn usage(&self) -> usize;

}

pub trait AbstractTranspositionTable {

    type EntryType: AbstractTTEntry;

    fn size(&self) -> ByteSize {
        ByteSize::from_bytes(self.internal_table().len() * size_of::<Self::EntryType>())
    }

    fn calculate_table_len(size: ByteSize) -> usize {
        size.bytes() / size_of::<Self::EntryType>()
    }

    fn internal_table(&self) -> &Vec<Self::EntryType>;

    fn internal_table_mut(&mut self) -> &mut Vec<Self::EntryType>;

    fn fetch_age(&self) -> u8;

    fn increase_age(&self);

    fn clear_age(&self);

    fn clear_mut(&self, threads: u32) {
        self.clear_age();

        if self.size().mib() < 32 {
            for entry in self.internal_table().iter() {
                entry.clear_mut();
            }
        } else {
            std::thread::scope(|s| {
                for chunk in self.internal_table().chunks(threads as usize) {
                    s.spawn(|| {
                        for entry in chunk.iter() {
                            entry.clear_mut();
                        }
                    });
                }
            });
        }
    }

    fn resize_mut(&mut self, size: ByteSize) {
        self.clear_age();

        let len = Self::calculate_table_len(size);

        *self.internal_table_mut() = Vec::new();

        unsafe {
            *self.internal_table_mut() = Vec::from_raw_parts(
                std::alloc::alloc_zeroed(
                    std::alloc::Layout::array::<Self::EntryType>(len).unwrap()
                ).cast(),
                len, len
            );
        };
    }

    fn hash_usage(&self) -> f64 {
        let samples: usize = self.internal_table().len().min(1000);

        let sum: usize = self.internal_table().iter()
            .take(samples)
            .map(Self::EntryType::usage)
            .sum();

        sum as f64 / (samples as u64 * Self::EntryType::BUCKET_SIZE) as f64 * 100.0
    }

    fn total_entries(&self) -> usize {
        self.internal_table().iter()
            .map(Self::EntryType::usage)
            .sum()
    }

}
