use crate::memo::slice_pattern_memo::SlicePatternMemo;
use crate::slice::SliceKey;
use crate::slice_pattern::SlicePattern;

#[derive(Default)]
pub struct BufferedSlicePatternMemo {
    temp: Vec<SlicePattern>
}

impl SlicePatternMemo for BufferedSlicePatternMemo {

    fn probe_or_put_mut<F>(&mut self, _key: SliceKey, produce: F) -> &SlicePattern
    where F: FnOnce() -> SlicePattern
    {
        if self.temp.len() > 100 {
            self.temp.remove(0);
        }

        self.temp.push(produce());
        self.temp.last().unwrap()
    }

}
