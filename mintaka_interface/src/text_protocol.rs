mod commandline_game_manager;

fn main() {
    loop {
        match "d" {
            "parse-board" => {
            }
            "show-board" => {
            }
            "clear-board" => {
            }
            "set" => {
            }
            "unset" => {
            }
            "batch-set" => {
            }
            "gen" => {
                // launch
            }
            "quite" => {
                break;
            }
            &_ => {
                println!("unknown command.");
            }
        }
    }
}
