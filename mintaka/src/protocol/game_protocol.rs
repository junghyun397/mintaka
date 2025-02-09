use crate::protocol::command::Command;
use crate::protocol::response::Response;

pub trait GameManager {

    fn response(&self, response: Response);

    fn command(&self, command: Command) -> Option<Response> {
        None
    }

}
