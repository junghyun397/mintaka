use mintaka::protocol::response::{Response, ResponseSender};

pub struct UnboundedResponseSender {
    tx: tokio::sync::mpsc::UnboundedSender<Response>,
}

impl ResponseSender for UnboundedResponseSender {
    fn response(&self, response: Response) {
        self.tx.send(response).unwrap();
    }
}

impl UnboundedResponseSender {
    pub fn new(tx: tokio::sync::mpsc::UnboundedSender<Response>) -> Self {
        Self { tx }
    }
}
