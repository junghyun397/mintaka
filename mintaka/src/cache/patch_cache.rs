use crate::slice_pattern::SlicePatch;

pub trait PatchCache {

    fn probe(&self, key: u32) -> Option<SlicePatch>;

    fn probe_mut(&mut self, key: u32) -> Option<SlicePatch>;

    fn put_mut(&mut self, key: u32, value: SlicePatch);

}
