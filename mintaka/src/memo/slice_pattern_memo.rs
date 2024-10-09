use crate::memo::hash_key::HashKey;
use crate::slice_pattern::SlicePattern;

pub trait SlicePatternMemo {

    fn probe_or_put_mut<F>(&mut self, key: HashKey, produce: F) -> &SlicePattern
    where F : FnOnce() -> SlicePattern;

}
