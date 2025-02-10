use crate::value::Depth;
use rusty_renju::notation::pos;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub max_vcf_depth: Depth,
}

impl Default for Config {

    fn default() -> Self {
        Config {
            max_vcf_depth: pos::U8_BOARD_SIZE,
        }
    }

}
