use crate::movegen::movegen_window::MovegenWindow;
use rusty_renju::board::Board;
use rusty_renju::history::History;
use rusty_renju::notation::pos::{MaybePos, Pos};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Default, Debug, Copy, Clone)]
pub struct GameState {
    pub board: Board,
    pub history: History,
    pub movegen_window: MovegenWindow,
}

impl GameState {

    pub fn from_board_and_history(board: Board, history: History) -> Self {
        GameState {
            board,
            history,
            movegen_window: MovegenWindow::from(&board.hot_field),
        }
    }

    pub fn set_mut(&mut self, pos: Pos) {
        self.board.set_mut(pos);
        self.history.set_mut(pos);

        self.movegen_window.imprint_window_mut(pos);
    }

    pub fn pass_mut(&mut self) {
        self.board.pass_mut();
        self.history.pass_mut();
    }

    pub fn unset_mut(&mut self, movegen_window: MovegenWindow) {
        self.movegen_window = movegen_window;

        match self.history.pop_mut().unwrap() {
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

}

impl Into<GameState> for Board {
    fn into(self) -> GameState {
        let history = (&self).try_into().unwrap_or(History::default());

        GameState {
            board: self,
            history,
            movegen_window: MovegenWindow::from(&self.hot_field),
        }
    }
}

impl Into<GameState> for History {
    fn into(self) -> GameState {
        let board = self.into();
        GameState {
            board,
            history: self,
            movegen_window: MovegenWindow::from(&board.hot_field),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GameStateData {
    board: Board,
    history: History,
}

impl Serialize for GameState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        GameStateData {
            board: self.board,
            history: self.history,
        }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GameState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let data = GameStateData::deserialize(deserializer)?;

        Ok(GameState {
            movegen_window: MovegenWindow::from(&data.board.hot_field),
            board: data.board,
            history: data.history,
        })
    }
}
