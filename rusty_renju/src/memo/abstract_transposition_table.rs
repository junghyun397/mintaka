pub trait AbstractTTEntry {

    const BUCKET_SIZE: usize;

    fn clear_mut(&self);

    fn usage(&self) -> usize;

}

pub trait AbstractTranspositionTable {

    type EntryType: AbstractTTEntry;

    fn calculate_table_len_in_kib(size_in_kib: usize) -> usize {
        size_in_kib * 1024 / size_of::<Self::EntryType>()
    }

    fn internal_table(&self) -> &Vec<Self::EntryType>;

    fn internal_table_mut(&mut self) -> &mut Vec<Self::EntryType>;

    fn fetch_age(&self) -> u8;

    fn increase_age(&self);

    fn clear_age(&self);

    fn clear_mut(&self) {
        self.clear_age();

        for entry in self.internal_table() {
            entry.clear_mut();
        }
    }

    fn size_in_kib(&self) -> usize {
        size_of_val(&self.internal_table()) / 1024
    }

    fn resize_mut(&mut self, size_in_kib: usize) {
        self.clear_age();

        let len = Self::calculate_table_len_in_kib(size_in_kib);

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

        sum as f64 / (samples * Self::EntryType::BUCKET_SIZE) as f64 * 100.0
    }

    fn total_entries(&self) -> usize {
        self.internal_table().iter()
            .map(Self::EntryType::usage)
            .sum()
    }

}
