use mintaka::protocol::game_manager::GameManager;
use mintaka::protocol::response::Response;

pub struct CommandLineGameManager;

impl GameManager for CommandLineGameManager {

    fn response(&self, response: Response) {
        match response {
            Response::Info(_) => {}
            Response::Warning(_) => {}
            Response::Error(_) => {}
            Response::Status(_) => {}
            Response::Board(_) => {}
            Response::BestMove(_, _) => {}
            Response::Switched => {}
            Response::Aborted => {}
        }
    }

}
