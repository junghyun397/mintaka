use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::slice::SliceKey;
use crate::slice_pattern::SlicePattern;
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct LruSlicePatternMemo {
    pub memo: LruCache<SliceKey, SlicePattern>,
}

impl Default for LruSlicePatternMemo {

    fn default() -> Self {
        Self {
            memo: LruCache::new(NonZeroUsize::new(10_000_000).unwrap())
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
