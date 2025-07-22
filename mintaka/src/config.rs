use rusty_renju::notation::pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SearchObjective {
    #[default] Best,
    Zeroing,
    Pondering
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReportContents {
    pub main_pv: bool,
}

impl Default for ReportContents {
    fn default() -> Self {
        ReportContents {
            main_pv: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Config {
    pub rule_kind: RuleKind,
    pub draw_condition: Option<usize>,

    pub search_objective: SearchObjective,

    pub max_nodes_in_1k: usize,
    pub max_depth: usize,
    pub max_vcf_depth: usize,

    pub tt_size: ByteSize,
    pub workers: NonZeroU32,

    pub time_management: bool,
    pub repost_contents: ReportContents,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rule_kind: RuleKind::Renju,
            draw_condition: None,
            search_objective: SearchObjective::default(),
            max_nodes_in_1k: usize::MAX,
            max_depth: usize::MAX,
            max_vcf_depth: pos::BOARD_SIZE - 5,
            tt_size: ByteSize::from_mib(16),
            workers: NonZeroU32::new(1).unwrap(),
            time_management: true,
            repost_contents: ReportContents::default(),
        }
    }
}
