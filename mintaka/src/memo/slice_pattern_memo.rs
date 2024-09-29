use crate::slice::SliceKey;
use crate::slice_pattern::SlicePattern;

pub trait SlicePatternMemo {

    fn probe_or_put_mut<F>(&mut self, key: SliceKey, produce: F) -> &SlicePattern
    where F : FnOnce() -> SlicePattern;

}
