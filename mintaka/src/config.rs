use rusty_renju::notation::pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::notation::value::Depth;
use std::num::NonZeroUsize;
use std::time::Duration;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchObjective {
    Best,
    Zeroing,
    Pondering
}

#[derive(Debug, Clone)]
pub enum SearchLimit {
    Time { turn_time: Duration },
    Nodes { in_1k: usize },
    Infinite,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub rule_kind: RuleKind,
    pub draw_condition: Option<Depth>,

    pub search_objective: SearchObjective,

    pub workers: NonZeroUsize,

    pub max_vcf_depth: Depth,

    pub report_search_status: bool,
    pub report_main_pv: bool,
}

impl Default for Config {

    fn default() -> Self {
        Config {
            rule_kind: RuleKind::Renju,
            draw_condition: None,
            search_objective: SearchObjective::Best,
            workers: NonZeroUsize::new(1).unwrap(),
            max_vcf_depth: pos::U8_BOARD_SIZE,
            report_search_status: false,
            report_main_pv: false,
        }
    }

}
