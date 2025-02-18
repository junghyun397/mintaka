use crate::search_limit::SearchLimit;
use rusty_renju::notation::pos;
use rusty_renju::notation::value::Depth;

#[derive(Default, Debug, Clone, Copy)]
pub enum PolicyType {
    #[default] Static,
    Confusion,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub search_limit: SearchLimit,
    pub policy_type: PolicyType,

    pub max_vcf_depth: Depth,
    pub max_ply: u8,
}

impl Default for Config {

    fn default() -> Self {
        Config {
            max_vcf_depth: pos::U8_BOARD_SIZE,
            max_ply: u8::MAX,
            search_limit: SearchLimit::Infinite,
            policy_type: PolicyType::Static,
        }
    }

}
