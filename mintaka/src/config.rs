use crate::value::Depth;
use rusty_renju::notation::pos;

pub struct Config {
    pub player_vcf_depth: Depth,
    pub ai_vcf_depth: Depth,

    pub max_search_depth: Depth,
    pub max_nodes: usize,

    pub time_limit_milliseconds: usize,

    pub workers: usize,
}

impl Default for Config {

    fn default() -> Self {
        Config {
            player_vcf_depth: pos::U8_BOARD_SIZE,
            ai_vcf_depth: pos::U8_BOARD_SIZE,
            max_search_depth: 100,
            max_nodes: 1_000_000,
            time_limit_milliseconds: usize::MAX,
            workers: 1,
        }
    }

}
