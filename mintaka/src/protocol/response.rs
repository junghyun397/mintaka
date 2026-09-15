use crate::principal_variation::PrincipalVariation;
use rusty_renju::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::score::Score;
use std::time::Duration;
use crate::protocol::nodes::Nodes;
use crate::protocol::time::{TimeUnit, TimeValue};
use crate::utils::depth::Depth;

#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde_with::skip_serializing_none)]
#[derive(Debug, Copy, Clone)]
pub struct ComputingResource {
    pub workers: u32,
    pub time_unit: TimeUnit,
    pub time_limit: Option<TimeValue>,
}

#[cfg(feature = "serde")]
#[cfg_attr(feature = "typeshare", typeshare::typeshare)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Response {
    Begins(ComputingResource),
    Status {
        hash: HashKey,
        best_move: MaybePos,
        score: Score,
        selective_depth: Depth,
        total_nodes: Nodes,
        pv: PrincipalVariation,
        time_elapsed: Duration,
    },
}

#[cfg(not(feature = "serde"))]
#[derive(Debug, Clone)]
pub enum Response {
    Begins(ComputingResource),
    Status {
        hash: HashKey,
        best_move: MaybePos,
        score: Score,
        selective_depth: Depth,
        total_nodes: Nodes,
        pv: PrincipalVariation,
        time_elapsed: Duration,
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
