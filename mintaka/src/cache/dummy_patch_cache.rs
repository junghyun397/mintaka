#![allow(unused_variables)]

use crate::cache::patch_cache::PatchCache;
use crate::slice::SliceKey;
use crate::slice_pattern::SlicePatch;

pub struct DummyPatchCache;

impl Default for DummyPatchCache {

    fn default() -> Self {
        DummyPatchCache
    }

}

impl PatchCache for DummyPatchCache {

    fn probe(&self, key: SliceKey) -> Option<SlicePatch> {
        None
    }

    fn probe_mut(&mut self, key: SliceKey) -> Option<SlicePatch> {
        None
    }

    fn put_mut(&mut self, key: SliceKey, _value: SlicePatch) {}

}
