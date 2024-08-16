use crate::notation::pos::Pos;

#[derive(Clone)]
pub struct History(pub Vec<Pos>);

impl History {

    pub fn empty() -> Self {
        History(Vec::new())
    }

}
