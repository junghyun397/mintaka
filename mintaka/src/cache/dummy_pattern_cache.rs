use crate::cache::pattern_cache::PatternCache;
use crate::slice::SliceKey;
use crate::slice_pattern::SlicePattern;

#[derive(Default)]
pub struct DummyPatternCache {}

impl PatternCache for DummyPatternCache {

    fn probe(&self, _key: SliceKey) -> Option<SlicePattern> {
        None
    }

    fn probe_mut(&mut self, _key: SliceKey) -> Option<SlicePattern> {
        None
    }

    fn put_mut(&mut self, _key: SliceKey, _value: SlicePattern) {}

}
