use std::time::Duration;
use crate::principal_variation::PrincipalVariation;
use crate::value::Depth;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::score::Score;
use serde::{Deserialize, Serialize};
use rusty_renju::utils::byte_size::ByteSize;

#[typeshare::typeshare]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ComputingResource {
    pub workers: u32,
    pub tt_size: ByteSize,
    #[typeshare(serialized_as = "DurationSchema")]
    pub time: Option<Duration>,
    #[typeshare(serialized_as = "Option<number>")]
    pub nodes_in_1k: Option<u64>,
}

#[typeshare::typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Response {
    Begins(ComputingResource),
    Status {
        hash: HashKey,
        best_move: MaybePos,
        score: Score,
        selective_depth: Depth,
        #[typeshare(serialized_as = "number")]
        total_nodes_in_1k: u64,
        pv: PrincipalVariation,
    },
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
