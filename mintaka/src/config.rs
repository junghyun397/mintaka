use crate::search_limit::SearchLimit;
use rusty_renju::notation::pos;
use rusty_renju::notation::value::Depth;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub max_vcf_depth: Depth,
    pub search_limit: SearchLimit,
}

impl Default for Config {

    fn default() -> Self {
        Config {
            max_vcf_depth: pos::U8_BOARD_SIZE,
            search_limit: SearchLimit::Infinite,
        }
    }

}
