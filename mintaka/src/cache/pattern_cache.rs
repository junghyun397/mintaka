use crate::slice_pattern::SlicePattern;

pub trait PatternCache {

    fn probe(&self, key: u32) -> Option<SlicePattern>;

    fn probe_mut(&mut self, key: u32) -> Option<SlicePattern>;

    fn put_mut(&mut self, key: u32, value: SlicePattern);

}
