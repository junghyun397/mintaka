use crate::game_agent::ComputingResource;
use crate::principal_variation::PrincipalVariation;
use crate::protocol::message;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Begins(ComputingResource),
    Status {
        eval: f32,
        total_nodes_in_1k: usize,
        best_moves: Vec<(Pos, Score)>,
        hash_usage: f32,
    },
    Pv(PrincipalVariation),
    Finished,
}

pub trait ResponseSender: Send {

    fn response(&self, response: Response);

}

#[derive(Clone)]
pub struct MpscResponseSender {
    sender: mpsc::Sender<Response>,
}

impl ResponseSender for MpscResponseSender {
    fn response(&self, response: Response) {
        self.sender.send(response).expect(message::CHANNEL_CLOSED_MESSAGE);
    }
}

impl MpscResponseSender {

    pub fn new(sender: mpsc::Sender<Response>) -> Self {
        Self { sender }
    }

}

pub struct NullResponseSender;

impl ResponseSender for NullResponseSender {
    fn response(&self, _response: Response) {}
}
