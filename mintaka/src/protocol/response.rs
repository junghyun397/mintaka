use crate::principal_variation::PrincipalVariation;
use crate::protocol::message;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::value::Score;
use rusty_renju::utils::byte_size::ByteSize;
use std::fmt::Display;
use std::sync::mpsc;
use std::time::Duration;

pub enum Response {
    Begins {
        workers: usize,
        running_time: Duration,
        tt_size: ByteSize,
    },
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
