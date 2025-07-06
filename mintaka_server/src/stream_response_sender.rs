use mintaka::protocol::response::{Response, ResponseSender};

pub struct StreamResponseSender {
    tx: tokio::sync::mpsc::UnboundedSender<Response>,
}

impl ResponseSender for StreamResponseSender {
    fn response(&self, response: Response) {
        self.tx.send(response).unwrap();
    }
}

impl StreamResponseSender {
    pub fn new(tx: tokio::sync::mpsc::UnboundedSender<Response>) -> Self {
        Self { tx }
    }
}
