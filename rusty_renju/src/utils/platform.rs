pub const U32_LANE_N: usize = 16;
pub const U8_LANE_N: usize = 64;

pub fn available_cores() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
}
