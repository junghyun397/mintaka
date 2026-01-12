use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::pos::{MaybePos, Pos};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use typeshare::typeshare;

#[typeshare(serialized_as = "GameStateData")]
#[derive(Default, Debug, Copy, Clone)]
pub struct GameState {
    pub board: Board,
    pub history: History,
    pub movegen_window: MovegenWindow,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct RecoveryState {
    pub movegen_window: MovegenWindow
}

impl RecoveryState {
    pub const EMPTY: Self = Self {
        movegen_window: MovegenWindow::EMPTY
    };
}

impl GameState {

    pub fn from_board_and_history(board: Board, history: History) -> Self {
        GameState {
            board,
            history,
            movegen_window: MovegenWindow::from(&board.hot_field),
        }
    }

    pub fn recovery_state(&self) -> RecoveryState {
        RecoveryState {
            movegen_window: self.movegen_window
        }
    }

    pub fn play(mut self, pos: Pos) -> Self {
        self.set_mut(pos);
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

    pub fn set_mut(&mut self, pos: Pos) {
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
        let last_action: MaybePos = self.history.pop_mut().into();
        match last_action {
            MaybePos::NONE => {
                self.board.unpass_mut();
            },
            pos => {
                let pos = pos.unwrap();

                self.board.unset_mut(pos);
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
        let history = (&board).try_into().unwrap_or_default();

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

#[typeshare::typeshare]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct GameStateData {
    board: Board,
    history: History,
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
