use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::bitfield::Bitfield;
use rusty_renju::board::{Board, MoveArtifact, UndoCache};
use rusty_renju::board_io::BoardData;
use rusty_renju::history::History;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::empty::Empty;
#[cfg(feature = "typeshare")]
use typeshare::typeshare;

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "GameStateData"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "GameStateData"))]
#[derive(Copy, Clone)]
pub struct GameState<const R: RuleKind> {
    pub board: Board<R>,
    pub history: History,
    pub movegen_window: MovegenWindow,
}

impl<const R: RuleKind> Empty for GameState<R> {
    fn empty() -> Self {
        Self {
            board: Board::empty(),
            history: History::empty(),
            movegen_window: MovegenWindow::default(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RecoveryState {
    pub movegen_field: Bitfield,
    pub unset_cache: UndoCache,
}

impl RecoveryState {
    pub const EMPTY: Self = Self {
        movegen_field: Bitfield::ZERO_FILLED,
        unset_cache: UndoCache::EMPTY,
    };
}

impl<const R: RuleKind> GameState<R> {
    pub fn play(mut self, pos: Pos) -> Self {
        self.play_mut(pos);
        self
    }

    pub fn pass(mut self) -> Self {
        self.pass_mut();
        self
    }

    pub fn undo(mut self, recovery_state: RecoveryState) -> Self {
        self.undo_mut(&recovery_state);
        self
    }

    pub fn undo_rebuild(mut self) -> Self {
        self.undo_rebuild_mut();
        self
    }

    pub fn play_mut(&mut self, pos: Pos) -> (MoveArtifact, RecoveryState) {
        let movegen_field = self.movegen_window.movegen_field;
        self.history.set_mut(pos);
        self.movegen_window.imprint_window(pos);

        let (artifact, unset_cache) = self.board.set_mut(pos);
        (artifact, RecoveryState { movegen_field, unset_cache })
    }

    pub fn pass_mut(&mut self) {
        self.board.pass_mut();
        self.history.pass_mut();
    }

    pub fn undo_mut(&mut self, recovery_state: &RecoveryState) -> MoveArtifact {
        self.movegen_window.movegen_field = recovery_state.movegen_field;

        if let Some(action) = self.history.pop_mut() {
            if let Some(pos) = action.ok() {
                return self.board.restore_mut(pos, &recovery_state.unset_cache);
            }

            self.board.unpass_mut();
        }

        MoveArtifact::empty()
    }

    pub fn undo_rebuild_mut(&mut self) -> MoveArtifact {
        let artifact = self.undo_move();
        self.movegen_window = MovegenWindow::from(&self.board.hot_field);
        
        artifact
    }

    fn undo_move(&mut self) -> MoveArtifact {
        if let Some(action) = self.history.pop_mut() {
            if let Some(pos) = action.ok() {
                return self.board.unset_mut(pos)
            } else {
                self.board.unpass_mut();
            }
        }

        MoveArtifact::empty()
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn is_empty(&self) -> bool {
        self.board.hot_field.is_empty()
    }
}

impl<const R: RuleKind> From<Board<R>> for GameState<R> {
    fn from(board: Board<R>) -> Self {
        let history: History = board.build_history();

        GameState {
            board,
            history,
            movegen_window: MovegenWindow::from(&board.hot_field),
        }
    }
}

impl<const R: RuleKind> From<History> for GameState<R> {
    fn from(history: History) -> Self {
        let board = (&history).into();

        GameState {
            board,
            history,
            movegen_window: MovegenWindow::from(&board.hot_field),
        }
    }
}

#[cfg_attr(feature = "typeshare", typeshare)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct GameStateData {
    pub board_data: BoardData,
    pub history: History,
}

impl<const R: RuleKind> From<&GameState<R>> for GameStateData {
    fn from(state: &GameState<R>) -> Self {
        Self { board_data: (&state.board).into(), history: state.history }
    }
}

impl<const R: RuleKind> From<GameStateData> for GameState<R> {
    fn from(data: GameStateData) -> Self {
        let Ok(board) = Board::try_from(data.board_data) else {
            panic!("invalid rule kind");
        };

        let movegen_window = MovegenWindow::from(&board.hot_field);

        Self {
            board,
            history: data.history,
            movegen_window,
        }
    }
}

#[cfg(feature = "serde")]
impl<const R: RuleKind> serde::Serialize for GameState<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        GameStateData::from(self).serialize(serializer)
    }
}
