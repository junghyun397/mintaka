use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;

pub trait EndgameAccumulator {

    const DISTANCE_WINDOW: u8;

    const ZERO: Self;

    fn unit(pos: Pos) -> Self;

    fn append_pos(self, defend: Pos, threat: Pos) -> Self;

    fn has_result(&self) -> bool;

}

pub type SequenceEndgameAccumulator = Option<Vec<Pos>>;

impl EndgameAccumulator for SequenceEndgameAccumulator {

    const DISTANCE_WINDOW: u8 = 7;

    const ZERO: Self = None;

    #[inline]
    fn unit(pos: Pos) -> Self {
        Some(vec![pos])
    }

    #[inline]
    fn append_pos(self, defend: Pos, four: Pos) -> Self {
        self.map(|mut sequence| {
            sequence.push(defend);
            sequence.push(four);
            sequence
        })
    }

    #[inline]
    fn has_result(&self) -> bool {
        self.is_some()
    }

}

impl EndgameAccumulator for Score {

    const DISTANCE_WINDOW: u8 = 5;

    const ZERO: Self = 0;

    #[inline]
    fn unit(_pos: Pos) -> Self {
        Score::MAX
    }

    #[inline]
    fn append_pos(self, _defend: Pos, _four: Pos) -> Self {
        self
    }

    #[inline]
    fn has_result(&self) -> bool {
        *self != 0
    }

}
