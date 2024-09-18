use crate::cache::patch_cache::PatchCache;
use crate::slice::SliceKey;
use crate::slice_pattern::SlicePattern;

#[derive(Default)]
pub struct DummyPatchCache;

impl PatchCache for DummyPatchCache {

    fn probe(&self, _key: SliceKey) -> Option<SlicePattern> {
        None
    }

    fn probe_mut(&mut self, _key: SliceKey) -> Option<SlicePattern> {
        None
    }

    fn put_mut(&mut self, _key: SliceKey, _value: SlicePattern) {}

}
