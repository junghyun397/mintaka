#![feature(adt_const_params)]

#[path = "../text_protocol.rs"]
mod text_protocol;

fn main() -> Result<(), impl std::error::Error> {
    text_protocol::entry::<{ rusty_renju::notation::rule::RuleKind::Gomoku }>()
}
