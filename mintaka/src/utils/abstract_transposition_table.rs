use crate::memo::hash_key::HashKey;

pub trait AbstractTranspositionTable<T> {

    fn internal_table(&self) -> &Vec<T>;

    fn assign_internal_table_mut(&mut self, table: Vec<T>);

    fn calculate_table_len_in_mib(size_in_mib: usize) -> usize {
        size_in_mib * 1024 * 1024 / size_of::<T>()
    }

    fn calculate_index(&self, key: HashKey) -> usize {
        ((key.0 as u128 * (self.internal_table().len() as u128)) >> 64) as usize
    }

    fn resize(&mut self, size_in_mib: usize) {
        let len = Self::calculate_table_len_in_mib(size_in_mib);
        unsafe {
            let new_table = Vec::from_raw_parts(
                std::alloc::alloc_zeroed(
                    std::alloc::Layout::array::<T>(len).unwrap()
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
            let element = &self.internal_table()[idx];
            _mm_prefetch::<_MM_HINT_T0>((element as *const T).cast());
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            use std::arch::aarch64::{_prefetch, _PREFETCH_LOCALITY0, _PREFETCH_READ};
            let idx = self.calculate_index(key);
            let entry = &self.internal_table()[idx];
            _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY0>((entry as *const T).cast());
        }
    }

}