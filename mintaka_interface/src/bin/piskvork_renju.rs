#![feature(adt_const_params)]

#[path = "../piskvork.rs"]
mod piskvork;

fn main() -> Result<(), impl std::error::Error> {
    piskvork::entry::<{ rusty_renju::notation::rule::RuleKind::Renju }>()
}
