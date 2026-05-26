use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::pos::{MaybePos, Pos};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, Deserializer, Serializer};
#[cfg(feature = "typeshare")]
use typeshare::typeshare;
use rusty_renju::utils::empty::Empty;

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "GameStateData"))]
#[derive(Debug, Copy, Clone)]
pub struct GameState {
    pub board: Board,
    pub history: History,
    pub movegen_window: MovegenWindow,
}

impl Empty for GameState {
    fn empty() -> Self {
        Self {
            board: Board::empty(),
            history: History::empty(),
            movegen_window: MovegenWindow::EMPTY,
        }
    }
}

#[cfg_attr(feature = "typeshare", typeshare)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct GameStateData {
    pub board: Board,
    pub history: History,
}

impl From<GameState> for GameStateData {
    fn from(state: GameState) -> Self {
        Self { board: state.board, history: state.history }
    }
}

impl From<GameStateData> for GameState {
    fn from(data: GameStateData) -> Self {
        let movegen_window = MovegenWindow::from(&data.board.hot_field);

        Self {
            board: data.board,
            history: data.history,
            movegen_window,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RecoveryState {
    pub movegen_window: MovegenWindow
}

impl RecoveryState {
    pub const EMPTY: Self = Self {
        movegen_window: MovegenWindow::EMPTY
    };
}

impl GameState {

    pub fn recovery_state(&self) -> RecoveryState {
        RecoveryState {
            movegen_window: self.movegen_window
        }
    }

    pub fn play(mut self, pos: Pos) -> Self {
        self.play_mut(pos);
        self
    }

    pub fn pass(mut self) -> Self {
        self.pass_mut();
        self
    }

    pub fn undo(mut self, recovery_state: RecoveryState) -> Self {
        self.undo_mut(recovery_state);
        self
    }

    pub fn undo_rebuild(mut self) -> Self {
        self.undo_rebuild_mut();
        self
    }

    pub fn play_mut(&mut self, pos: Pos) {
        self.board.set_mut(pos);
        self.history.set_mut(pos);

        self.movegen_window.imprint_window(pos);
    }

    pub fn pass_mut(&mut self) {
        self.board.pass_mut();
        self.history.pass_mut();
    }

    pub fn undo_mut(&mut self, recovery_state: RecoveryState) {
        self.movegen_window = recovery_state.movegen_window;

        self.undo_move();
    }

    pub fn undo_rebuild_mut(&mut self) {
        self.undo_move();
        self.movegen_window = MovegenWindow::from(&self.board.hot_field);
    }

    fn undo_move(&mut self) {
        if let Some(action) = self.history.pop_mut() {
            if let Some(pos) = action.ok() {
                self.board.unset_mut(pos);
            } else {
                self.board.unpass_mut();
            }
        }
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn is_empty(&self) -> bool {
        self.board.hot_field.is_empty()
    }

}

impl From<Board> for GameState {
    fn from(board: Board) -> Self {
        let history: History = (&board).try_into().unwrap_or_else(|_| History::empty());

        GameState {
            board,
            history,
            movegen_window: MovegenWindow::from(&board.hot_field),
        }
    }
}

impl From<History> for GameState {
    fn from(history: History) -> Self {
        let board = (&history).into();

        GameState {
            board,
            history,
            movegen_window: MovegenWindow::from(&board.hot_field),
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for GameState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        GameStateData {
            board: self.board,
            history: self.history,
        }.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for GameState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let data = GameStateData::deserialize(deserializer)?;

        Ok(GameState {
            movegen_window: MovegenWindow::from(&data.board.hot_field),
            board: data.board,
            history: data.history,
        })
    }
}
