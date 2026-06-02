use rusty_renju::notation::rule::RuleKind;

#[cfg(feature = "rule-renju")]
pub const RULE: RuleKind = RuleKind::Renju;

#[cfg(feature = "rule-gomoku")]
pub const RULE: RuleKind = RuleKind::Gomoku;

#[cfg(feature = "clap")]
pub mod preference;
pub mod message;
