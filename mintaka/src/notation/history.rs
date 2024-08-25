use crate::notation::pos::Pos;

#[derive(Clone)]
pub struct History(pub Vec<Pos>);

impl Default for History {

    fn default() -> Self {
        Self(Vec::new())
    }

}

impl History {

    pub fn play(&self, pos: Pos) -> Self {
        let mut history = self.0.clone();
        history.push(pos);

        Self(history)
    }

    pub fn undo(&self) -> Self {
        let mut history = self.0.clone();
        history.pop();

        Self(history)
    }

    pub fn play_mut(&mut self, pos: Pos) {
        self.0.push(pos);
    }

    pub fn undo_mut(&mut self) {
        self.0.pop();
    }

}
