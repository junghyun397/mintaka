use crate::notation::pos::Pos;

#[derive(Clone, Default)]
pub struct History(pub Vec<Option<Pos>>);

impl History {

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, idx: usize) -> Option<Pos> {
        self.0.get(idx)
            .copied()
            .flatten()
    }

    pub fn play_mut(&mut self, pos: Pos) {
        self.0.push(Some(pos));
    }

    pub fn undo_mut(&mut self) -> Option<Pos> {
        self.0.pop().unwrap_or(None)
    }

    pub fn pass_mut(&mut self) {
        self.0.push(None)
    }

}
