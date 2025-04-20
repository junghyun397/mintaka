// use wider lane for instruction level parallelism

#[cfg(any(target_feature = "avx2", target_feature = "avx512f"))]
pub const U32_LANE_N: usize = 32; // 225 % 32 = 1
#[cfg(not(all(target_feature = "avx2", target_feature = "avx512f")))]
pub const U32_LANE_N: usize = 16;

pub const U8_LANE_N: usize = {
    if cfg!(target_feature = "avx512f") {
        64
    } else if cfg!(target_feature = "avx2") {
        32
    } else {
        16
    }
};
