use crate::notation::color::Color;
use crate::notation::pos::Pos;
use crate::opening::opening_kind::OpeningKind;

pub enum OpeningStage {
    Move(OpeningMove),
    Swap(OpeningSwap),
    Declare(OpeningDeclare),
    Offer(OpeningOffer),
    Select(OpeningSelect),
    Branch(OpeningBranch)
}

pub fn new_agent(opening_kind: OpeningKind) -> OpeningStage {
    OpeningStage::Move(OpeningMove {
        moves: 1,
        opening_kind,
        move_window_width: 3,
    })
}

pub trait OpeningAgent {

    fn moves(&self) -> usize;

    fn opening_kind(&self) -> OpeningKind;

}

pub trait MoveStageOpeningAgent : OpeningAgent {

    fn validate_move(&self, pos: Pos) -> bool;

}

macro_rules! impl_opening_agent {
    ($name:ident) => {
        impl OpeningAgent for $name {

            fn moves(&self) -> usize {
                self.moves
            }

            fn opening_kind(&self) -> OpeningKind {
                self.opening_kind
            }

        }
    };
}

pub struct OpeningMove {
    moves: usize,
    opening_kind: OpeningKind,
    pub move_window_width: usize,
}

impl_opening_agent!(OpeningMove);

impl MoveStageOpeningAgent for OpeningMove {

    fn validate_move(&self, pos: Pos) -> bool {
        todo!()
    }

}

impl OpeningMove {

    fn set(&self, pos: Pos) -> Option<OpeningStage> {
        match self.opening_kind {
            OpeningKind::Soosyrv8 => panic!(),
            OpeningKind::Taraguchi10 => panic!(),
        }
    }

}

pub struct OpeningSwap {
    moves: usize,
    opening_kind: OpeningKind,
    pub color: Color,
}

impl_opening_agent!(OpeningSwap);

impl OpeningSwap {

    fn swap(&self, do_swap: bool) -> OpeningStage {
        match self.opening_kind {
            OpeningKind::Soosyrv8 => todo!(),
            OpeningKind::Taraguchi10 => todo!()
        }
    }

}

pub struct OpeningDeclare {
    moves: usize,
    opening_kind: OpeningKind,
    pub min_candidates: usize,
    pub max_candidates: usize,
}

impl_opening_agent!(OpeningDeclare);

impl OpeningDeclare {
    
    fn declare(&self, count: usize) -> Option<OpeningStage> {
        match self.opening_kind {
            OpeningKind::Soosyrv8 => todo!(),
            OpeningKind::Taraguchi10 => todo!()
        }
    }

}

pub struct OpeningOffer {
    moves: usize,
    opening_kind: OpeningKind,
    pub remaining_moves: usize,
    pub symmetry_moves: Box<[Pos]>,
    pub offers: Vec<Pos>,
}

impl_opening_agent!(OpeningOffer);

impl MoveStageOpeningAgent for OpeningOffer {

    fn validate_move(&self, pos: Pos) -> bool {
        todo!()
    }

}

impl OpeningOffer {

    fn add(&self, pos: Pos) -> Option<OpeningStage> {
        match self.opening_kind {
            OpeningKind::Soosyrv8 => todo!(),
            OpeningKind::Taraguchi10 => todo!()
        }
    }

}

pub struct OpeningSelect {
    moves: usize,
    opening_kind: OpeningKind,
    pub offered_moves: Box<[Pos]>,
}

impl_opening_agent!(OpeningSelect);

impl MoveStageOpeningAgent for OpeningSelect {

    fn validate_move(&self, pos: Pos) -> bool {
        match self.opening_kind {
            OpeningKind::Soosyrv8 => todo!(),
            OpeningKind::Taraguchi10 => todo!()
        }
    }

}

impl OpeningSelect {

    fn select(&self, pos: Pos) -> Option<OpeningSelect> {
        match self.opening_kind {
            OpeningKind::Soosyrv8 => todo!(),
            OpeningKind::Taraguchi10 => todo!()
        }
    }

}

pub struct OpeningBranch {
    moves: usize,
    opening_kind: OpeningKind,
}

impl_opening_agent!(OpeningBranch);

impl OpeningBranch {

    fn branch(&self, make_offer: bool) -> OpeningStage {
        match self.opening_kind {
            OpeningKind::Soosyrv8 => panic!(),
            OpeningKind::Taraguchi10 => todo!()
        }
    }

}
