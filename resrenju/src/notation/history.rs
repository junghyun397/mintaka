use crate::notation::pos::Pos;

pub struct History(pub Vec<Pos>);

impl History {

    pub fn empty() -> Self {
        History(Vec::new())
    }

}
