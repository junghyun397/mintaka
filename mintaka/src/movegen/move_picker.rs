use crate::eval::evaluator::Evaluator;
use crate::game_state::GameState;
use crate::movegen::move_generator;
use crate::movegen::move_list::{MoveEntry, MoveList};
use crate::thread_data::ThreadData;
use crate::thread_type::ThreadType;
use rusty_renju::bitfield::Bitfield;
use rusty_renju::notation::pos::{MaybePos, Pos};
use rusty_renju::utils::empty::Empty;
use std::cmp::PartialEq;
use rusty_renju::notation::rule::RuleKind;
use crate::thread_data;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ThreatKind {
    ForkFour(Bitfield),
    ForkThreeFour(Bitfield),
}

impl ThreatKind {
    fn bitfield(&self) -> &Bitfield {
        match self {
            ThreatKind::ForkFour(field) => field,
            ThreatKind::ForkThreeFour(field) => field,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum MoveStage {
    TT,
    Killer,
    Generate(MoveKind),
    Moves(MoveKind),
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum MoveKind {
    All,
    ThreatDirect,
    ExtendFour
}

pub struct MovePicker<const R: RuleKind> {
    threat_kind: Option<ThreatKind>,
    stage: MoveStage,
    moves_buffer: MoveList,
    tt_move: MaybePos,
    killer_moves: [MaybePos; thread_data::KILLER_MOVE_SLOTS],
    occupied_moves: Bitfield,
    skip_lp_quiets: bool,
}

impl<const R: RuleKind> MovePicker<R> {
    pub fn init_new(
        tt_move: MaybePos,
        killer_moves: [MaybePos; thread_data::KILLER_MOVE_SLOTS],
        threat_kind: Option<ThreatKind>,
    ) -> Self {
        Self {
            threat_kind,
            stage: MoveStage::TT,
            moves_buffer: MoveList::empty(),
            tt_move,
            killer_moves,
            occupied_moves: Bitfield::empty(),
            skip_lp_quiets: false,
        }
    }

    pub fn skip_lp_quiets(&mut self) {
        self.skip_lp_quiets = true;
    }

    pub fn next(
        &mut self,
        td: &mut ThreadData<R, impl ThreadType, impl Evaluator<R>>,
        state: &GameState<R>,
    ) -> Option<MoveEntry> {
        loop {
            match self.stage {
                MoveStage::TT => {
                    self.stage = MoveStage::Killer;

                    if let Some(tt_move) = self.tt_move.ok()
                        && self.is_forced_legal(state, tt_move)
                    {
                        self.occupied_moves.set(tt_move);

                        return Some(MoveEntry {
                            pos: tt_move,
                            move_score: move_generator::TT_MOVE_SCORE,
                            lp_quiet: false,
                            history_score: None,
                        });
                    }
                }
                MoveStage::Killer => {
                    loop {
                        let Some(killer_move) = self.killer_moves[0].ok() else {
                            match self.threat_kind {
                                Some(_) => {
                                    self.stage = MoveStage::Generate(MoveKind::ThreatDirect)
                                },
                                None => {
                                    self.stage = MoveStage::Generate(MoveKind::All)
                                }
                            }

                            break;
                        };

                        self.killer_moves[0] = self.killer_moves[1];
                        self.killer_moves[1] = MaybePos::NONE;

                        if self.occupied_moves.is_cold(killer_move)
                            && self.is_forced_legal(state, killer_move)
                        {
                            self.occupied_moves.set(killer_move);

                            return Some(MoveEntry {
                                pos: killer_move,
                                move_score: move_generator::KILLER_MOVE_SCORE,
                                lp_quiet: false,
                                history_score: None,
                            });
                        }
                    }
                }
                MoveStage::Generate(kind) => {
                    match kind {
                        MoveKind::All =>
                            move_generator::generate_all_moves(&mut self.moves_buffer, td, state),
                        MoveKind::ThreatDirect =>
                            move_generator::generate_threat_direct_response(&mut self.moves_buffer, td, state, self.threat_kind.unwrap().bitfield()),
                        MoveKind::ExtendFour =>
                            move_generator::generate_extend_four_response(&mut self.moves_buffer, td, state),
                    }

                    self.stage = MoveStage::Moves(kind);
                }
                MoveStage::Moves(kind) => {
                    while let Some(next_move) = self.moves_buffer.consume_best() {
                        if self.skip_lp_quiets && next_move.lp_quiet {
                            return None
                        }

                        if self.occupied_moves.is_hot(next_move.pos) {
                            continue;
                        }

                        self.occupied_moves.set(next_move.pos);

                        return Some(next_move);
                    }

                    match kind {
                        MoveKind::ThreatDirect => self.stage = MoveStage::Generate(MoveKind::ExtendFour),
                        _ => return None,
                    }
                }
            }
        }
    }

    fn is_forced_legal(&self, state: &GameState<R>, pos: Pos) -> bool {
        if let Some(threat_kind) = self.threat_kind {
            threat_kind.bitfield().is_hot(pos)
                || state.board.patterns.field[state.board.player_color][pos.idx_usize()].has_closed_four()
        } else {
            true
        }
    }
}
