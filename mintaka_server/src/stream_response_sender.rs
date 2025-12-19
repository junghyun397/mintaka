use crate::session::SessionResponse;
use mintaka::protocol::response::{Response, ResponseSender};

#[derive(Clone)]
pub struct StreamSessionResponseSender {
    tx: tokio::sync::mpsc::UnboundedSender<SessionResponse>,
}

impl StreamSessionResponseSender {
    pub fn new(tx: tokio::sync::mpsc::UnboundedSender<SessionResponse>) -> Self {
        Self { tx }
    }
}

impl ResponseSender for StreamSessionResponseSender {
    fn response(&self, response: Response) {
        let _ = self.tx.send(SessionResponse::Response(response));
    }
}
