pub const U32_WIDE_LANE_N: usize = 64;

pub const U32_LANE_N: usize = {
    if cfg!(target_feature = "avx512f") {
        16
    } else if cfg!(target_feature = "avx2") {
        8
    } else {
        4
    }
};

pub const U32_REGISTER_N: usize = {
    if cfg!(target_feature = "avx512f") {
        16
    } else if cfg!(target_feature = "avx2") {
        4
    } else {
        16
    }
};

pub const U32_TOTAL_LANES: usize = U32_LANE_N * U32_REGISTER_N;

pub const U8_LANE_N: usize = {
    if cfg!(target_feature = "avx512f") {
        64
    } else if cfg!(target_feature = "avx2") {
        32
    } else {
        16
    }
};

pub const U8_REGISTER_N: usize = {
    if cfg!(target_feature = "avx512f") {
        4
    } else if cfg!(target_feature = "avx2") {
        8
    } else {
        16
    }
};
