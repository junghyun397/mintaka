use rusty_renju::notation::score::Score;
use crate::utils::depth::Depth;

macro_rules! parse_int {
    ($name:literal,$t:ty,$default:expr) => {
        parse_or_default!($name,$t,1.0,$default)
    };
}

macro_rules! parse_float {
    ($name:literal,$t:ty,$default:expr) => {
        parse_or_default!($name,$t,0.001,$default)
    };
}

macro_rules! parse_or_default {
    ($name:literal,$t:ty,$scale:expr,$default:expr) => {{
        match option_env!($name) {
            Some(value) => match i64::from_str_radix(value, 10) {
                Ok(value) => (value as f64 * $scale) as $t,
                Err(_) => $default,
            },
            None => $default,
        }
    }};
}

pub const ASPIRATION_DELTA_BASE: i32 = parse_int!("aspiration_delta_base", i32, 8);
pub const ASPIRATION_DELTA_DIV: i32 = parse_int!("aspiration_delta_div", i32, 8192);

pub const LMR_BASE: f64 = parse_float!("lmr_base", f64, 0.8);
pub const LMR_DIV: f64 = parse_float!("lmr_div", f64, 2.4);

pub const LMP_BASE: usize = parse_int!("lmp_base", usize, 2);
pub const LMP_DIV_IMPROVING: f64 = parse_float!("lmp_div_improving", f64, 1.0);
pub const LMP_DIV_NON_IMPROVING: f64 = parse_float!("lmp_div_non_improving", f64, 2.0);

pub const FP_BASE: i32 = parse_int!("fp_base", i32, 100);
pub const FP_MUL: i32 = parse_int!("fp_mul", i32, 32);

pub const RAZORING_MAX_DEPTH_LEFT: Depth = Depth::from_i32(parse_int!("razoring_max_depth", i32, 2));
pub const RAZORING_MARGIN: [Score; 5] =
    [
        Score::from_i32(parse_int!("razoring_margin_0", i32, 200)),
        Score::from_i32(parse_int!("razoring_margin_1", i32, 320)),
        Score::from_i32(parse_int!("razoring_margin_2", i32, 480)),
        Score::from_i32(parse_int!("razoring_margin_3", i32, 710)),
        Score::from_i32(parse_int!("razoring_margin_4", i32, 1000)),
    ];

pub const HT_QUIET_BONUS_MUL: i32 = parse_int!("ht_quiet_bonus_mul", i32, 4);
pub const HT_TACTICAL_BONUS_MUL: i32 = parse_int!("ht_tactical_bonus_mul", i32, 4);
pub const HT_AGEING_MUL: f64 = parse_float!("ht_ageing_mul", f64, 0.75);
