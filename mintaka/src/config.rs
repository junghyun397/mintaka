use rusty_renju::notation::pos;
use rusty_renju::notation::rule::RuleKind;
use std::num::NonZeroUsize;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum SearchObjective {
    #[default] Best,
    Zeroing,
    Pondering
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub rule_kind: RuleKind,
    pub draw_condition: Option<usize>,

    pub search_objective: SearchObjective,

    pub max_nodes_in_1k: usize,
    pub max_depth: usize,
    pub max_vcf_depth: usize,

    pub workers: NonZeroUsize,


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
            workers: NonZeroUsize::new(1).unwrap(),
            repost_contents: ReportContents::default(),
        }
    }
}
