use crate::game_agent::ComputingResource;
use crate::principal_variation::PrincipalVariation;
use crate::value::Depth;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::value::Score;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Begins(ComputingResource),
    Status {
        best_move: MaybePos,
        score: Score,
        pv: PrincipalVariation,
        total_nodes_in_1k: usize,
        depth: Depth,
    },
    Finished,
}

pub trait ResponseSender: Send {

    fn response(&self, response: Response);

}

#[derive(Clone)]
pub struct CallBackResponseSender<F> where F: Fn(Response) + Send {
    consumer: F,
}

impl<F> ResponseSender for CallBackResponseSender<F> where F: Fn(Response) + Send {
    fn response(&self, response: Response) {
        (self.consumer)(response);
    }
}

impl<F> CallBackResponseSender<F> where F: Fn(Response) + Send {

    pub fn new(consumer: F) -> Self {
        Self { consumer }
    }

}

pub struct NullResponseSender;

impl ResponseSender for NullResponseSender {
    fn response(&self, _response: Response) {}
}
