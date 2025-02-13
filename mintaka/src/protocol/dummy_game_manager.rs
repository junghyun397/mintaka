use crate::protocol::game_manager::GameManager;
use crate::protocol::response::Response;

pub struct DummyGameManager; impl GameManager for DummyGameManager {

    fn response(&self, response: Response) {
        println!("{:?}", response);
    }

}
