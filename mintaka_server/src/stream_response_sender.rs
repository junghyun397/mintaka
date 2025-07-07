use crate::session::SessionKey;
use mintaka::game_agent::{BestMove, GameAgent};
use mintaka::protocol::response::{Response, ResponseSender};

pub struct StreamResponseSender<T> {
    session_key: SessionKey,
    tx: tokio::sync::mpsc::UnboundedSender<(SessionKey, T)>,
}

impl<T> StreamResponseSender<T> {
    pub fn new(session_key: SessionKey, tx: tokio::sync::mpsc::UnboundedSender<(SessionKey, T)>) -> Self {
        Self { session_key, tx }
    }
}

impl ResponseSender for StreamResponseSender<Response> {
    fn response(&self, response: Response) {
        self.tx.send((self.session_key, response)).unwrap();
    }
}

impl StreamResponseSender<(GameAgent, BestMove)> {
    pub fn send(&self, game_agent: GameAgent, best_move: BestMove) {
        self.tx.send((self.session_key, (game_agent, best_move))).unwrap();
    }
}
