use rusty_renju::notation::pos::Pos;
use std::iter::Map;
use std::rc::Rc;

pub struct Solution {
    pub solution: Pos,
    pub child: Option<Map<Pos, Rc<Solution>>>
}

impl TryFrom<Box<[u8]>> for Solution {

    type Error = ();

    fn try_from(value: Box<[u8]>) -> Result<Self, Self::Error> {
        todo!()
    }

}

impl Solution {

    pub fn to_bson_binary(&self) -> Box<[u8]> {
        todo!()
    }

}
