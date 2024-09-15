use crate::cache::patch_cache::PatchCache;
use crate::slice_pattern::SlicePatch;

pub struct LruPatchCache {

}

impl Default for LruPatchCache {

    fn default() -> Self {
        LruPatchCache {}
    }

}

impl PatchCache for LruPatchCache {

    fn probe(&self, key: u32) -> Option<SlicePatch> {
        todo!()
    }

    fn probe_mut(&mut self, key: u32) -> Option<SlicePatch> {
        todo!()
    }

    fn put_mut(&mut self, key: u32, value: SlicePatch) {
        todo!()
    }

}
