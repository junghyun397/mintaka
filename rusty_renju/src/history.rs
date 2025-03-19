use crate::notation::pos::Pos;

#[derive(Copy, Clone)]
pub enum Action {
    Move(Pos),
    Pass
}

impl Action {
    
    pub fn maybe_move(&self) -> Option<Pos> {
        match self {
            Action::Move(pos) => Some(*pos),
            Action::Pass => None
        }
    }
    
    pub fn unwrap(&self) -> Pos {
        match self {
            Action::Move(pos) => *pos,
            Action::Pass => unreachable!()
        }
    }

}

#[derive(Clone, Default)]
pub struct History(pub Vec<Action>);

impl History {

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, idx: usize) -> Option<Action> {
        self.0.get(idx)
            .copied()
    }

    pub fn play_mut(&mut self, pos: Pos) {
        self.0.push(Action::Move(pos));
    }

    pub fn undo_mut(&mut self) -> Action {
        self.0.pop().unwrap_or(Action::Pass)
    }

    pub fn pass_mut(&mut self) {
        self.0.push(Action::Pass)
    }

    pub fn pop_mut(&mut self) -> Option<Action> {
        self.0.pop()
    }

}
