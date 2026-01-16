use std::time::Duration;
use crate::principal_variation::PrincipalVariation;
use crate::value::Depth;
use rusty_renju::memo::hash_key::HashKey;
use rusty_renju::notation::pos::MaybePos;
use rusty_renju::notation::score::Score;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "typeshare")]
use typeshare::typeshare;
use rusty_renju::utils::byte_size::ByteSize;

#[cfg_attr(feature = "typeshare", typeshare)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone)]
pub struct ComputingResource {
    pub workers: u32,
    pub tt_size: ByteSize,
    #[cfg_attr(feature = "typeshare", typeshare(serialized_as = "DurationSchema"))]
    pub time: Option<Duration>,
    pub nodes_in_1k: Option<u32>,
}

#[cfg_attr(feature = "typeshare", typeshare(serialized_as = "ResponseSchema"))]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "type", content = "content"),
)]
#[derive(Debug, Clone)]
pub enum Response {
    Begins(ComputingResource),
    Status {
        hash: HashKey,
        best_move: MaybePos,
        score: Score,
        selective_depth: Depth,
        total_nodes_in_1k: u32,
        pv: PrincipalVariation,
    },
}

#[cfg(any())]
mod typeshare_workaround {
    use super::*;
    #[cfg_attr(feature = "typeshare", typeshare)]
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type", content = "content")]
    pub enum ResponseSchema {
        Begins(ComputingResource),
        Status {
            hash: HashKey,
            best_move: MaybePos,
            score: Score,
            selective_depth: Depth,
            total_nodes_in_1k: u32,
            pv: PrincipalVariation,
        },
    }
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
