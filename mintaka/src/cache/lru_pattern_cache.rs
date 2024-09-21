use crate::cache::pattern_cache::PatternCache;
use crate::slice_pattern::SlicePattern;

pub struct LruPatternCache {

}

impl Default for LruPatternCache {

    fn default() -> Self {
        LruPatternCache {}
    }

}

impl PatternCache for LruPatternCache {

    fn probe(&self, key: u32) -> Option<SlicePattern> {
        todo!()
    }

    fn probe_mut(&mut self, key: u32) -> Option<SlicePattern> {
        todo!()
    }

    fn put_mut(&mut self, key: u32, value: SlicePattern) {
        todo!()
    }

}
