use crate::search_limit::SearchLimit;
use rusty_renju::notation::pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::value::Depth;
use std::num::NonZeroUsize;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchObjective {
    #[default] Best,
    Zeroing,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub rule_kind: RuleKind,
    pub search_objective: SearchObjective,

    pub workers: NonZeroUsize,
    pub search_limit: SearchLimit,

    pub max_vcf_depth: Depth,
    pub max_ply: u8,
}

impl Default for Config {

    fn default() -> Self {
        Config {
            rule_kind: RuleKind::Renju,
            search_objective: SearchObjective::Best,
            workers: NonZeroUsize::new(1).unwrap(),
            search_limit: SearchLimit::Infinite,
            max_vcf_depth: pos::U8_BOARD_SIZE,
            max_ply: u8::MAX,
        }
    }

}
