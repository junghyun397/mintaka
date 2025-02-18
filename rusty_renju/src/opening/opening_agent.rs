use crate::game::Game;
use crate::history::History;
use crate::notation::color::Color;
use crate::notation::pos;
use crate::notation::pos::Pos;
use crate::opening::opening_utils::find_forbidden_symmetry_moves;
use std::collections::HashSet;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum OpeningKind {
    Soosyrv8,
    Taraguchi10,
    Zeroed,
}

pub enum OpeningStage {
    Move(OpeningMove),
    Swap(OpeningSwap),
    Declare(OpeningDeclare),
    Offer(OpeningOffer),
    Select(OpeningSelect),
    Branch(OpeningBranch),
    Finish
}

pub fn new_agent(opening_kind: OpeningKind) -> OpeningStage {
    OpeningStage::Move(OpeningMove {
        moves: 0,
        opening_kind,
        opener_color: Color::Black,
        move_window_width: 1,
    })
}

pub trait OpeningAgent {

    fn moves(&self) -> usize;

    fn opening_kind(&self) -> OpeningKind;

    fn opener_color(&self) -> Color;

    fn opponent_color(&self) -> Color {
        !self.opener_color()
    }

    fn openers_turn(&self) -> bool {
        self.opener_color() == Color::player_color_from_moves(self.moves())
    }

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

            fn opener_color(&self) -> Color {
                self.opener_color
            }

        }
    };
}

pub struct OpeningMove {
    moves: usize,
    opening_kind: OpeningKind,
    opener_color: Color,
    pub move_window_width: u8,
}

impl_opening_agent!(OpeningMove);

impl MoveStageOpeningAgent for OpeningMove {

    fn validate_move(&self, pos: Pos) -> bool {
        let pole: u8 = pos::BOARD_WIDTH / 2 - self.move_window_width / 2;
        let (row, col) = pos.to_cartesian();

        (pole <= row && row < pole + self.move_window_width)
            && (pole <= col && col < pole + self.move_window_width)
    }

}

impl OpeningMove {

    fn set(&self, game: Game, pos: Pos) -> (Game, Option<OpeningStage>) {
        if self.validate_move(pos) {
            let game = game.play(pos);

            let next = match self.opening_kind {
                OpeningKind::Soosyrv8 => match self.moves {
                    0 ..= 3 => OpeningStage::Move(OpeningMove {
                        opening_kind: self.opening_kind,
                        opener_color: if self.moves % 2 == 1 { Color::White } else { Color::Black },
                        moves: self.moves,
                        move_window_width: (self.moves * 2 - 1) as u8,
                    }),
                    4 => OpeningStage::Swap(OpeningSwap {
                        moves: self.moves + 1,
                        opening_kind: self.opening_kind,
                        opener_color: self.opener_color,
                        offer_count: None,
                        color: Color::Black,
                    }),
                    _ => OpeningStage::Finish
                },
                OpeningKind::Taraguchi10 => match self.moves {
                    0 ..= 3 => OpeningStage::Swap(OpeningSwap {
                        moves: self.moves,
                        opening_kind: self.opening_kind,
                        opener_color: self.opener_color,
                        offer_count: None,
                        color: Color::Black,
                    }),
                    4 => OpeningStage::Branch(OpeningBranch {
                        moves: 4,
                        opening_kind: self.opening_kind,
                        opener_color: self.opener_color,
                    }),
                    _ => OpeningStage::Finish
                },
                OpeningKind::Zeroed => unreachable!()
            };

            (game, Some(next))
        } else {
            (game, None)
        }
    }

}

pub struct OpeningSwap {
    moves: usize,
    opening_kind: OpeningKind,
    opener_color: Color,
    pub offer_count: Option<usize>,
    pub color: Color,
}

impl_opening_agent!(OpeningSwap);

impl OpeningSwap {

    fn swap(&self, do_swap: bool) -> OpeningStage {
        let opener_color = if do_swap {
            !self.opener_color
        } else {
            self.opener_color
        };

        match self.opening_kind {
            OpeningKind::Soosyrv8 => match self.offer_count {
                None => OpeningStage::Move(OpeningMove {
                    moves: self.moves,
                    opening_kind: self.opening_kind,
                    opener_color,
                    move_window_width: pos::BOARD_WIDTH,
                }),
                Some(offer_count) => OpeningStage::Offer(OpeningOffer {
                    moves: self.moves,
                    opening_kind: self.opening_kind,
                    opener_color,
                    total_moves: offer_count,
                    remaining_moves: offer_count,
                    symmetry_moves: HashSet::new(),
                    offers: vec![],
                }),
            },
            OpeningKind::Taraguchi10 => match self.moves {
                0 ..= 4 => OpeningStage::Move(OpeningMove {
                    moves: self.moves,
                    opening_kind: self.opening_kind,
                    opener_color,
                    move_window_width: 0,
                }),
                _ => OpeningStage::Finish
            },
            OpeningKind::Zeroed => unreachable!()
        }
    }

}

pub struct OpeningDeclare {
    moves: usize,
    opening_kind: OpeningKind,
    opener_color: Color,
    pub min_candidates: usize,
    pub max_candidates: usize,
}

impl_opening_agent!(OpeningDeclare);

impl OpeningDeclare {
    
    fn declare(&self, count: usize) -> Option<OpeningStage> {
        match self.opening_kind() {
            OpeningKind::Soosyrv8 => {
                (self.min_candidates ..= self.max_candidates).contains(&count).then(||
                    OpeningStage::Swap(OpeningSwap {
                        moves: self.moves,
                        opening_kind: self.opening_kind,
                        opener_color: self.opener_color,
                        offer_count: Some(count),
                        color: Color::Black,
                    }),
                )
            }
            OpeningKind::Taraguchi10 => unreachable!(),
            OpeningKind::Zeroed => unreachable!(),
        }
    }

}

pub struct OpeningOffer {
    moves: usize,
    opening_kind: OpeningKind,
    opener_color: Color,
    pub total_moves: usize,
    pub remaining_moves: usize,
    pub symmetry_moves: HashSet<Pos>,
    pub offers: Vec<Pos>,
}

impl_opening_agent!(OpeningOffer);

impl MoveStageOpeningAgent for OpeningOffer {

    fn validate_move(&self, pos: Pos) -> bool {
        self.symmetry_moves.contains(&pos)
    }

}

impl OpeningOffer {

    fn add(&self, history: History, pos: Pos) -> Option<OpeningStage> {
        self.validate_move(pos).then(|| {
            let mut offers = self.offers.clone();
            offers.push(pos);

            if self.remaining_moves == 1 {
                OpeningStage::Select(OpeningSelect {
                    moves: self.moves,
                    opening_kind: self.opening_kind,
                    opener_color: self.opener_color,
                    offered_moves: offers.into(),
                })
            } else {
                let mut symmetry_moves = self.symmetry_moves.to_owned();
                symmetry_moves.extend(find_forbidden_symmetry_moves(&history, pos));

                OpeningStage::Offer(OpeningOffer {
                    moves: self.moves,
                    opening_kind: self.opening_kind,
                    opener_color: self.opener_color,
                    total_moves: self.total_moves,
                    remaining_moves: self.remaining_moves - 1,
                    symmetry_moves,
                    offers,
                })
            }
        })
    }

}

pub struct OpeningSelect {
    moves: usize,
    opening_kind: OpeningKind,
    opener_color: Color,
    pub offered_moves: Box<[Pos]>,
}

impl_opening_agent!(OpeningSelect);

impl MoveStageOpeningAgent for OpeningSelect {

    fn validate_move(&self, pos: Pos) -> bool {
        self.offered_moves.contains(&pos)
    }

}

impl OpeningSelect {

    fn select(&self, pos: Pos) -> Option<OpeningStage> {
        self.validate_move(pos).then(||
            match self.opening_kind {
                OpeningKind::Soosyrv8 => OpeningStage::Finish,
                OpeningKind::Taraguchi10 => OpeningStage::Finish,
                OpeningKind::Zeroed => OpeningStage::Finish,
            }
        )
    }

}

pub struct OpeningBranch {
    moves: usize,
    opening_kind: OpeningKind,
    opener_color: Color,
}

impl_opening_agent!(OpeningBranch);

impl OpeningBranch {

    fn branch(&self, make_offer: bool) -> OpeningStage {
        match self.opening_kind {
            OpeningKind::Taraguchi10 =>
                if make_offer {
                    OpeningStage::Swap(OpeningSwap {
                        moves: 4,
                        opening_kind: self.opening_kind,
                        opener_color: self.opener_color,
                        offer_count: None,
                        color: Color::Black,
                    })
                } else {
                    OpeningStage::Offer(OpeningOffer {
                        moves: 4,
                        opening_kind: self.opening_kind,
                        opener_color: self.opener_color,
                        total_moves: 10,
                        remaining_moves: 10,
                        symmetry_moves: HashSet::new(),
                        offers: vec![],
                    })
                },
            OpeningKind::Soosyrv8 => unreachable!(),
            OpeningKind::Zeroed => unreachable!(),
        }
    }

}
