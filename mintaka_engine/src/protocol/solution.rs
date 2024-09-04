use mintaka::notation::pos::Pos;
use std::iter::Map;

pub struct Solution {
    pub solution: Pos,
    pub child: Option<Map<Pos, Solution>>
}

impl Solution {

    pub fn from_bson_binary(source: Box<[u8]>) -> Self {
        todo!()
    }

    pub fn to_bson_binary(&self) -> Box<[u8]> {
        todo!()
    }

}
