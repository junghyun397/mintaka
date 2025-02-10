use mintaka::protocol::command::Command;
use mintaka::protocol::response::Response;

struct PbrainGameManager {} impl PbrainGameManager {

    fn response(&self, response: Response) {
        println!("{:?}", response);
    }

    fn command(&self, command: Command) -> Option<Response> {
        None
    }

}
