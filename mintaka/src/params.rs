use crate::value::Depth;
use rusty_renju::notation::score::Score;

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

pub const ASPIRATION_DELTA_BASE: Score = parse_int!("aspiration_delta_base", Score, 8);
pub const ASPIRATION_DELTA_DIV: Score = parse_int!("aspiration_delta_div", Score, 8192);

pub const LMR_BASE: f64 = parse_float!("lmr_base", f64, 0.8);
pub const LMR_DIV: f64 = parse_float!("lmr_div", f64, 2.4);

pub const LMP_BASE: usize = parse_int!("lmp_base", usize, 2);
pub const LMP_DIV_IMPROVING: f64 = parse_float!("lmp_div_improving", f64, 1.0);
pub const LMP_DIV_NON_IMPROVING: f64 = parse_float!("lmp_div_non_improving", f64, 2.0);

pub const FP_BASE: Depth = parse_int!("fp_base", Depth, 100);
pub const FP_MUL: Depth = parse_int!("fp_mul", Depth, 32);

pub const HISTORY_TABLE_AGEING_MUL: f64 = parse_float!("history_table_ageing_mul", f64, 0.75);
