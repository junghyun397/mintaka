use crate::memo::hash_key::HashKey;

pub trait AbstractTranspositionTable<T> {

    #[inline(always)]
    fn internal_table(&self) -> &Vec<T>;

    #[inline(always)]
    fn assign_internal_table_mut(&mut self, table: Vec<T>);

    #[inline(always)]
    fn calculate_index_u128(&self, key: u128) -> usize {
        ((key * (self.internal_table().len() as u128)) >> 64) as usize
    }

    fn resize(&mut self, size_in_mib: usize) {
        let size_in_bytes = size_in_mib * 1024 * 1024 / size_of::<T>();
        unsafe {
            self.assign_internal_table_mut(Vec::from_raw_parts(
                std::alloc::alloc_zeroed(
                    std::alloc::Layout::array::<T>(size_in_bytes).unwrap()
                ).cast(),
                size_in_bytes,
                size_in_bytes
            ));
        };
    }

    fn prefetch(&self, key: HashKey) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
            let idx = self.calculate_index_u128(key.0 as u128);
            let element = &self.internal_table()[idx];
            _mm_prefetch::<_MM_HINT_T0>((element as *const T).cast());
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            use std::arch::aarch64::{_prefetch, _PREFETCH_LOCALITY0, _PREFETCH_READ};
            let idx = self.calculate_index(key);
            let element = &self.internal_table()[idx];
            _prefetch::<_PREFETCH_READ, _PREFETCH_LOCALITY0>((element as *const T).cast());
        }
    }

}
