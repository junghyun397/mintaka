use crate::memo::hash_key::HashKey;

pub trait AbstractTTEntry {

    const BUCKET_SIZE: usize;

    fn clear_mut(&self);

    fn usage(&self) -> usize;

}

pub trait AbstractTranspositionTable {

    type EntryType: AbstractTTEntry;

    fn calculate_table_len_in_mib(size_in_mib: usize) -> usize {
        size_in_mib * 1024 * 1024 / size_of::<Self::EntryType>()
    }

    fn internal_table(&self) -> &Vec<Self::EntryType>;

    fn internal_table_mut(&mut self) -> &mut Vec<Self::EntryType>;

    fn assign_internal_table_mut(&mut self, table: Vec<Self::EntryType>);

    fn calculate_index(&self, key: HashKey) -> usize {
        ((key.0 as u128 * (self.internal_table().len() as u128)) >> 64) as usize
    }

    fn clear_mut(&self) {
        for entry in self.internal_table() {
            entry.clear_mut();
        }
    }

    fn resize_mut(&mut self, size_in_mib: usize) {
        let len = Self::calculate_table_len_in_mib(size_in_mib);
        unsafe {
            let new_table = Vec::from_raw_parts(
                std::alloc::alloc_zeroed(
                    std::alloc::Layout::array::<Self::EntryType>(len).unwrap()
                ).cast(),
                len,
                len
            );

            self.assign_internal_table_mut(new_table);
        };
    }

    fn prefetch(&self, key: HashKey) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
            let idx = self.calculate_index(key);
            let entry = &self.internal_table()[idx];
            _mm_prefetch::<_MM_HINT_T0>((entry as *const Self::EntryType).cast());
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            use std::arch::aarch64::{_prefetch, _PREFETCH_LOCALITY0, _PREFETCH_READ};
            let idx = self.calculate_index(key);
            let entry = &self.internal_table()[idx];
            _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY0>((entry as *const Self::EntryType).cast());
        }
    }

    fn hash_usage(&self) -> f64 {
        const SAMPLES: usize = 2000;

        let sum: usize = self.internal_table().iter()
            .take(self.internal_table().len().min(SAMPLES))
            .map(Self::EntryType::usage)
            .sum();

        sum as f64 / (SAMPLES * Self::EntryType::BUCKET_SIZE) as f64 * 100f64
    }

    fn total_entries(&self) -> usize {
        self.internal_table().iter()
            .map(Self::EntryType::usage)
            .sum()
    }

}
