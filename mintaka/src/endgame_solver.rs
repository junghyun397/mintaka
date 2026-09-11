use crate::config::{Config, SearchObjective};
use crate::eval::evaluator::{ActiveEvaluator, Evaluator};
use crate::game_state::GameState;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::protocol::response::NullResponseSender;
use crate::protocol::timer::Timer;
use crate::search_endgame;
use crate::search_endgame::ThreatSearchKind;
use crate::thread_data::ThreadData;
use crate::thread_type::MainThread;
use crate::time_manager::TimeManager;
use crate::utils::depth::Depth;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use rusty_renju::utils::empty::Empty;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::time::{Duration, Instant};

pub struct EndgameSolution {
    pub sequence: Option<Vec<Pos>>,
    pub elapsed: Duration,
    pub nodes: u32,
}

pub fn solve_endgame<const R: RuleKind>(
    mut state: GameState<R>,
    threat_kind: ThreatSearchKind,
    depth_limit: Option<Depth>,
    global_counter_1k: Arc<AtomicU32>,
    aborted: Arc<AtomicBool>,
) -> EndgameSolution {
    let config = Config {
        draw_condition: None,
        max_nodes_in_1k: None,
        max_depth: None,
        max_quiescence_depth: depth_limit,
        tt_size: ByteSize::from_kib(32),
        workers: 1,
        pondering: false,
        initial_timer: Timer::INFINITE,
        spawn_depth_specialist: false,
    };
    
    let tt = TranspositionTable::new_with_size(config.tt_size);
    let ht = HistoryTable::empty();

    let evaluator = ActiveEvaluator::from_state(&state);
    
    let start_time = Instant::now();

    let mut td = ThreadData::new(
        MainThread::new(
            NullResponseSender,
            TimeManager::init(Timer::INFINITE, start_time)
        ), 0,
        SearchObjective::Best, config,
        evaluator,
        tt.view(),
        ht,
        &aborted,
        &global_counter_1k,
    );
    
    let sequence = match threat_kind {
        ThreatSearchKind::VCF => search_endgame::endgame_proof::<R, { ThreatSearchKind::VCF }>(&mut td, &mut state),
        ThreatSearchKind::VCT => search_endgame::endgame_proof::<R, { ThreatSearchKind::VCT }>(&mut td, &mut state),
        ThreatSearchKind::Forced => search_endgame::endgame_proof::<R, { ThreatSearchKind::Forced }>(&mut td, &mut state),
    };
    
    let elapsed = start_time.elapsed();

    EndgameSolution { sequence, elapsed, nodes: td.batch_counter.count_local_in_1k() }
}
