use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::slice::SliceKey;
use crate::slice_pattern::SlicePattern;
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct LruSlicePatternMemo {
    memo: LruCache<SliceKey, SlicePattern>,
}

impl Default for LruSlicePatternMemo {

    fn default() -> Self {
        const SIZE_IN_MIB: usize = 256; // 1 MiB = 30,000 slice patterns
        const SIZE: usize = SIZE_IN_MIB * 1024 * 1024 / size_of::<SlicePattern>();
        Self {
            memo: LruCache::new(NonZeroUsize::new(SIZE).unwrap())
        }
    }

}

impl SlicePatternMemo for LruSlicePatternMemo {

    fn probe_or_put_mut<F>(&mut self, key: SliceKey, produce: F) -> &SlicePattern
    where F: FnOnce() -> SlicePattern
    {
        self.memo.get_or_insert(key, produce)
    }

}
