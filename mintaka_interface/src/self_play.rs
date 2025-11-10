use mintaka::config::{Config, SearchObjective};
use mintaka::game_agent::{ComputingResource, GameAgent, GameError};
use mintaka::game_state::GameState;
use mintaka::protocol::command::Command;
use mintaka::protocol::response::{CallBackResponseSender, Response};
use mintaka_interface::message::{Message, MessageSender};
use mintaka_interface::preference::Preference;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc};

fn main() -> Result<(), GameError> {
    let pref = Preference::parse();

    self_play(pref.default_config, pref.game_state.unwrap_or_else(|| GameState::default()))
}

fn response_printer(response: Response) {
    match response {
        Response::Begins(ComputingResource { workers, time, nodes_in_1k, tt_size }) =>
            println!("begins: workers={workers}, \
                running-time={time:?}, \
                nodes={nodes_in_1k:?}, \
                tt-size={tt_size}"),
        Response::Status { best_move, score, pv, total_nodes_in_1k, depth } =>
            println!(
                "status: depth={depth}, \
                score={score}, \
                best_move={best_move}, \
                total_nodes_in_1k={total_nodes_in_1k}, \
                pv={pv:?}"
            ),
        _ => {}
    }
}

fn self_play(config: Config, game_state: GameState) -> Result<(), GameError> {
    let aborted = Arc::new(AtomicBool::new(false));

    let (message_sender, message_receiver) = {
        let (tx, rx) = mpsc::channel();
        (MessageSender::new(tx), rx)
    };

    let mut game_agent = GameAgent::from_state(config, game_state);

    game_agent.command(Command::Workers(num_cpus::get_physical() as u32))?;

    message_sender.launch(SearchObjective::Best);

    let mut overall_nodes_in_1k = 0;
    let mut game_result = None;
    for message in message_receiver {
        match message {
            Message::Launch(search_objective) => {
                let best_move = game_agent.launch(
                    search_objective,
                    CallBackResponseSender::new(response_printer),
                    aborted.clone()
                );

                let result = game_agent.command(Command::Play(best_move.pos))?;

                println!("{}",
                         game_agent.state.board.to_string_with_last_moves(game_agent.state.history.recent_action_pair())
                );

                println!(
                    "solution: pos={}, score={}, depth={}, nodes={}k, elapsed={:?}, pv={:?}",
                    best_move.pos, best_move.score, best_move.selective_depth, best_move.total_nodes_in_1k, best_move.time_elapsed, best_move.pv,
                );

                overall_nodes_in_1k += best_move.total_nodes_in_1k;

                message_sender.result(result);
                message_sender.launch(SearchObjective::Best);
            },
            Message::Finished(result) => {
                game_result = Some(result);
                break;
            },
            _ => {}
        }
    }

    println!("{}", game_result.unwrap());
    println!("total {}k nodes", overall_nodes_in_1k);

    Ok(())
}
