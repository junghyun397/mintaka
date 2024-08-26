use crate::notation::pos::Pos;

#[derive(Clone)]
pub struct History(pub Vec<Option<Pos>>);

impl Default for History {

    fn default() -> Self {
        Self(Vec::new())
    }

}

impl History {

    pub fn get(&self, idx: usize) -> Option<Pos> {
        self.0.get(idx)
            .map(|x| *x)
            .flatten()
    }

    pub fn play(&self, pos: Option<Pos>) -> Self {
        let mut history = self.0.clone();
        history.push(pos);

        Self(history)
    }

    pub fn undo(&self) -> Self {
        let mut history = self.0.clone();
        history.pop();

        Self(history)
    }

    pub fn pass(&self) -> Self {
        let mut history = self.0.clone();
        history.push(None);

        Self(history)
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
