use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::slice_pattern::SlicePattern;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct HashMapSlicePatternMemo {
    table: HashMap<u64, SlicePattern>,
}

impl SlicePatternMemo for HashMapSlicePatternMemo {

    fn prefetch_memo(&self, _packed_slice: u64) {}

    fn probe_or_put_mut<F>(&mut self, packed_slice: u64, produce: F) -> SlicePattern
    where F: FnOnce() -> SlicePattern
    {
        let maybe_entry = self.table.get_mut(&packed_slice);

        if let Some(entry) = maybe_entry {
            *entry
        } else {
            let slice_pattern = produce();

            self.table.insert(packed_slice, slice_pattern);

            slice_pattern
        }

    }

}
