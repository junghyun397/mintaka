use crate::unbounded_response_sender::UnboundedResponseSender;
use mintaka::config::Config;
use mintaka::game_agent::{BestMove, GameAgent};
use mintaka::protocol::response::{Response, ResponseSender};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;

pub struct Session {
    game_agent: Option<GameAgent>,
}

impl Session {

    pub fn new(config: Config) -> Self {
        Self {
            game_agent: Some(GameAgent::new(config)),
        }
    }

    pub fn launch(&mut self) -> (UnboundedReceiver<Response>, tokio::task::JoinHandle<(GameAgent, BestMove)>) {
        let (response_sender, response_receiver) = {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            (UnboundedResponseSender::new(tx), rx)
        };

        let aborted = Arc::new(AtomicBool::new(false));
        let mut game_agent = std::mem::take(&mut self.game_agent).unwrap();

        let handle = tokio::task::spawn_blocking(move || {
            let best_move = game_agent.launch(response_sender, aborted.clone());

            (game_agent, best_move)
        });

        (response_receiver, handle)
    }

    pub fn recover(&mut self, game_agent: GameAgent) {
        self.game_agent = Some(game_agent);
    }

}
