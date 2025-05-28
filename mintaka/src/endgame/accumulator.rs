use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;

pub trait EndgameAccumulator {

    const DISTANCE_WINDOW: isize;

    const ZERO: Self;

    fn unit(pos: Pos, score: Score) -> Self;

    fn append_pos(self, defend: Pos, threat: Pos) -> Self;

}

pub type SequenceEndgameAccumulator = Option<Vec<Pos>>;

impl EndgameAccumulator for SequenceEndgameAccumulator {

    const DISTANCE_WINDOW: isize = 5;

    const ZERO: Self = None;

    #[inline]
    fn unit(pos: Pos, _score: Score) -> Self {
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

}

impl EndgameAccumulator for Score {

    const DISTANCE_WINDOW: isize = 5;

    const ZERO: Self = 0;

    #[inline]
    fn unit(_pos: Pos, score: Score) -> Self {
        score
    }

    #[inline]
    fn append_pos(self, _defend: Pos, _four: Pos) -> Self {
        self
    }

}
