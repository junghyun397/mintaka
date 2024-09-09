use crate::notation::pos::Pos;

#[derive(Clone)]
pub struct History(pub Vec<Option<Pos>>);

impl Default for History {

    fn default() -> Self {
        Self(Vec::new())
    }

}

impl History {

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, idx: usize) -> Option<Pos> {
        self.0.get(idx)
            .map(|x| *x)
            .flatten()
    }

    pub fn play_mut(&mut self, pos: Pos) {
        self.0.push(Some(pos));
    }

    pub fn undo_mut(&mut self) {
        self.0.pop();
    }

    pub fn pass_mut(&mut self) {
        self.0.push(None)
    }

}
