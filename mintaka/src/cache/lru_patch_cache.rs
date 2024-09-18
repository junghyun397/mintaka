use crate::cache::patch_cache::PatchCache;
use crate::slice_pattern::SlicePattern;

pub struct LruPatchCache {

}

impl Default for LruPatchCache {

    fn default() -> Self {
        LruPatchCache {}
    }

}

impl PatchCache for LruPatchCache {

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
